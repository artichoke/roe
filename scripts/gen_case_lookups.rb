#!/usr/bin/env ruby
# frozen_string_literal: true

require 'open3'

def unstaged_changes?
  _, _, status = Open3.capture3('git', 'diff', '--quiet')
  !status.success?
end

def generate_case_mapping(ucd_dir)
  output, error, status = Open3.capture3(
    'ucd-generate',
    'case-mapping',
    ucd_dir,
    '--include',
    'TITLE',
    '--flat-table'
  )
  abort error unless status.success?

  output
end

repo = File.expand_path('..', __dir__)
Dir.chdir(repo)

abort 'Stage your changes before running this task.' if unstaged_changes?

case_mapping = generate_case_mapping(File.join('generated', 'ucd'))
File.binwrite(File.join('generated', 'case_mapping.rs'), case_mapping)
exec('cargo', 'clippy', '--fix', '--allow-dirty')
