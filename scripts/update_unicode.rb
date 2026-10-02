#!/usr/bin/env ruby
# frozen_string_literal: true

# Per the Unicode Terms of Use -- https://www.unicode.org/copyright.html --
# data found under `https://www.unicode.org/Public/` are considered Unicode
# Data Files and are subject to these constraints:
#
# - Except where otherwise more broadly permitted or licensed:
#   - you may not make copies of or modifications to Unicode Products for
#     public distribution, or incorporate Unicode Products in whole or in part
#     into any product or publication, or otherwise publicly distribute them,
#     without the express written permission of Unicode, and
#   - you may not copy or extract fonts or font data from any Unicode Products,
#     including but not limited to Unicode Code Charts.
# - All Unicode Data Files and Unicode Software are subject to the terms and
#   conditions of the free and open-source Unicode License v3, unless otherwise
#   indicated by specific restriction, permission, or license identified at the
#   point of release or in such software, data file, or other documentation.
#
# The Unicode License v3, which can be found at <https://www.unicode.org/license.txt>
# is included in this repository. The license requires one of:
#
# (a) this copyright and permission notice appear with all copies of the Data
#     Files or Software
# (b) this copyright and permission notice appear in associated Documentation
#
# `roe` distributes this license as `LICENSE-UNICODE` in crate bundles and
# includes `AND Unicode-3.0` in the `Cargo.toml` SPDX license expression.
# See: https://spdx.org/licenses/Unicode-3.0.html.
#
# Updates to Unicode Data Files performed by this script also update the
# embedded license.

require 'fileutils'
require 'open-uri'

class UnicodeUpdater
  LATEST_README_URL = 'https://www.unicode.org/Public/UCD/latest/ucd/ReadMe.txt'
  LICENSE_URL = 'https://www.unicode.org/license.txt'
  UCD_FILES = %w[UnicodeData SpecialCasing CaseFolding ReadMe PropList].freeze

  def initialize(repo:)
    @repo = repo
  end

  def update
    version = latest_final_version
    FileUtils.mkdir_p(File.join(@repo, 'generated', 'ucd'))
    downloads(version).each_pair { |url, destination| download(url, destination) }
  end

  private

  def latest_final_version
    # Resolve the latest final release once so all inputs use the same version.
    readme = URI.open(LATEST_README_URL, &:read)
    version = readme[/final data files for version (\d+\.\d+\.\d+)/, 1]
    raise 'Could not identify a final Unicode release.' unless version

    version
  end

  def downloads(version)
    files = { LICENSE_URL => 'LICENSE-UNICODE' }
    UCD_FILES.each do |name|
      files["https://www.unicode.org/Public/#{version}/ucd/#{name}.txt"] = "generated/ucd/#{name}.txt"
    end
    files
  end

  def download(url, destination)
    URI.open(url) do |data|
      IO.copy_stream(data, File.join(@repo, destination))
    end
  end
end

UnicodeUpdater.new(repo: File.expand_path('..', __dir__)).update if $PROGRAM_NAME == __FILE__
