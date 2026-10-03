#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'
require 'open-uri'
require 'optparse'
require 'rbconfig'
require 'set'
require 'tmpdir'

# Parse raw UCD releases independently of Roe's production table generator.
# These maps determine version exclusions, never the expected fixture results.
class FixtureUnicodeData
  FILES = %w[ReadMe UnicodeData SpecialCasing CaseFolding].freeze

  attr_reader :version, :codepoints, :mappings

  def initialize(directory:)
    @directory = directory
    readme = File.read(path('ReadMe'))
    @version = readme[/\bversion\s+(\d+\.\d+\.\d+)/i, 1]
    raise 'Missing final Unicode version.' unless @version && readme.include?('final data files')

    %w[SpecialCasing CaseFolding].each do |name|
      raise "Mismatched #{name} version." unless File.foreach(path(name)).first.include?("#{name}-#{@version}.txt")
    end
    @codepoints = Set.new
    @mappings = Array.new(6) { {} } # lower, upper, title, fold, swap, Turkic fold
    load_unicode_data
    load_special_casing
    load_folding
    load_swapping
  end

  def changed_codepoints(other)
    (@codepoints | other.codepoints).select do |code|
      @mappings.zip(other.mappings).any? { |left, right| left.fetch(code, [code]) != right.fetch(code, [code]) }
    end.to_set
  end

  private

  def path(name)
    File.join(@directory, "#{name}.txt")
  end

  def sequence(value)
    value.split.map { |code| code.to_i(16) }
  end

  def load_unicode_data
    @title_components = {}
    File.foreach(path('UnicodeData')) do |line|
      fields = line.chomp.split(';', -1)
      code = fields[0].to_i(16)
      @codepoints << code if [12, 13, 14].any? { |column| !fields[column].empty? }
      [13, 12, 14].each_with_index do |column, kind|
        target = fields[column]
        target = fields[12] if kind == 2 && target.empty?
        @mappings[kind][code] = sequence(target) unless target.empty?
      end
      next unless fields[2] == 'Lt'

      @title_components[code] = sequence(fields[5].sub(/\A<[^>]+>\s*/, ''))
    end
  end

  def each_record(name)
    File.foreach(path(name)) do |line|
      data = line.split('#', 2).first.strip
      next if data.empty?

      fields = data.split(';', -1).map(&:strip)
      @codepoints << fields[0].to_i(16)
      yield fields
    end
  end

  def load_special_casing
    each_record('SpecialCasing') do |fields|
      next unless fields[4].empty?

      [1, 3, 2].each_with_index { |column, kind| @mappings[kind][fields[0].to_i(16)] = sequence(fields[column]) }
    end
  end

  def load_folding
    each_record('CaseFolding') do |fields|
      code = fields[0].to_i(16)
      case fields[1]
      when 'C', 'F'
        @mappings[3][code] = sequence(fields[2])
      when 'T'
        @mappings[5][code] = sequence(fields[2])
      end
    end
    @mappings[5] = @mappings[3].merge(@mappings[5])
  end

  def swapped(code)
    lower = @mappings[0].fetch(code, [code])
    lower == [code] ? @mappings[1].fetch(code, [code]) : lower
  end

  def load_swapping
    (@mappings[0].keys | @mappings[1].keys).each { |code| @mappings[4][code] = swapped(code) }
    @title_components.each { |code, parts| @mappings[4][code] = parts.flat_map { |part| swapped(part) } }
  end
end

class MRIFixtureGenerator
  OPERATIONS = {
    downcase: [[], [:ascii], [:turkic], [:lithuanian], [:fold]],
    upcase: [[], [:ascii], [:turkic], [:lithuanian]],
    capitalize: [[], [:ascii], [:turkic], [:lithuanian]],
    swapcase: [[], [:ascii], [:turkic], [:lithuanian]]
  }.freeze
  STRINGS = [
    '', 'HELLO WORLD', 'ßﬃ ABC', '1ABC', 'ΣΟΣ', "ΑΣ\u{301}", "I\u{307}",
    'İIıi', "iI\u{307}", "\u{301}ABC", 'ᾈᾀ', 'ǅǈǋǲ', 'Ꭰꭰ', 'İßΣς',
    'KȺ', 'ԱԲԳև', "\0ABC"
  ].freeze

  def self.run(arguments)
    repo = File.expand_path('..', __dir__)
    options = { ucd: File.join(repo, 'generated', 'ucd'), output: File.join(repo, 'tests', 'fixtures', "mri-#{RUBY_VERSION}-case-mapping.tsv") }
    parser = OptionParser.new do |cli|
      cli.banner = 'Usage: ruby scripts/gen_mri_fixtures.rb [options]'
      cli.on('--ucd DIR', 'Bundled target UCD (default: generated/ucd)') { |value| options[:ucd] = value }
      cli.on('--oracle-ucd DIR', 'Local UCD matching MRI; otherwise download versioned files') { |value| options[:oracle_ucd] = value }
      cli.on('--output FILE', 'Output fixture path') { |value| options[:output] = value }
      cli.on('-h', '--help', 'Show usage') { puts cli; return }
    end
    parser.parse!(arguments)
    raise "Unexpected arguments: #{arguments.join(' ')}" unless arguments.empty?
    raise 'Run this generator under MRI Ruby.' unless RUBY_ENGINE == 'ruby'

    new(**options).generate
  end

  def initialize(ucd:, output:, oracle_ucd: nil)
    @ucd = ucd
    @output = output
    @oracle_ucd = oracle_ucd
    @oracle_version = RbConfig::CONFIG.fetch('UNICODE_VERSION')
  end

  def generate
    if @oracle_ucd
      write_fixture(@oracle_ucd)
    else
      Dir.mktmpdir('roe-mri-ucd-') do |directory|
        download_oracle_data(directory)
        write_fixture(directory)
      end
    end
  end

  private

  def download_oracle_data(directory)
    FixtureUnicodeData::FILES.each do |name|
      url = "https://www.unicode.org/Public/#{@oracle_version}/ucd/#{name}.txt"
      URI.open(url) { |data| IO.copy_stream(data, File.join(directory, "#{name}.txt")) }
    end
  end

  def write_fixture(oracle_directory)
    target = FixtureUnicodeData.new(directory: @ucd)
    oracle = FixtureUnicodeData.new(directory: oracle_directory)
    raise "MRI needs UCD #{@oracle_version}, got #{oracle.version}." unless oracle.version == @oracle_version

    changed = oracle.changed_codepoints(target)
    inputs = (target.codepoints | (0..127).to_set).sort.map { |code| code.chr(Encoding::UTF_8) } + STRINGS
    retained = inputs.reject { |input| input.codepoints.any? { |code| changed.include?(code) } }
    # Capture every expectation directly from MRI. Do not consult Roe's output
    # when deciding which inputs to retain; mapping mismatches must fail tests.
    rows = retained.map { |input| fixture_row(input) }
    header = "# MRI #{RUBY_VERSION}; Unicode #{@oracle_version}; see README.md for columns and Unicode-version filtering."
    FileUtils.mkdir_p(File.dirname(@output))
    File.write(@output, ([header] + rows).join("\n") + "\n")
    warn "MRI #{RUBY_VERSION}; UCD #{@oracle_version} vs #{target.version}; #{changed.size} version-changed scalars; " \
         "#{inputs.size - retained.size} excluded inputs; #{retained.size} inputs; #{retained.size * 17} comparisons."
  end

  def fixture_row(input)
    results = OPERATIONS.flat_map do |method, modes|
      modes.map { |options| input.public_send(method, *options).unpack1('H*') }
    end
    ([input.unpack1('H*')] + results).join("\t")
  end
end

MRIFixtureGenerator.run(ARGV) if $PROGRAM_NAME == __FILE__
