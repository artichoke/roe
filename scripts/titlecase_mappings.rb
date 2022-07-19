#!/usr/bin/env ruby
# frozen_string_literal: true

mappings = {}

# Iterate ASCII range
("\0".."\x7F").each do |ch|
  ch_title = ch.capitalize
  mappings[ch] = ch_title unless ch == ch_title
end.count

# Iterate the remainder of Unicode.
#
# See `char::MAX` defined here:
#
# - https://doc.rust-lang.org/stable/std/char/constant.MAX.html
("\u{80}".."\u{10ffff}").each do |ch|
  ch_title = ch.capitalize
  mappings[ch] = ch_title unless ch == ch_title
end.count


if mappings.each_value.any? { |ch| ch.bytesize > 5 }
  source, title = mappings.each_pair.max_by { |ch, tc| tc.bytesize }
  warn "roe expects titlecase for all chars to yield no more than 6 bytes"
  warn "found a character that violates this assumption"
  warn ""
  warn "source = '#{source}', bytesize = #{source.bytesize}, bytes = #{source.bytes.inspect}"
  warn "title = '#{title}', bytesize = #{title.bytesize}, bytes = #{title.bytes.inspect}"
  exit 1
end
