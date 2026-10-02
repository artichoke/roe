# MRI case mapping fixture

`mri-4.0.7-case-mapping.tsv` contains results from MRI Ruby 4.0.7, which bundles
Unicode 17.0.0. Inputs cover the union of case-mapped codepoints in Roe's UCD,
all ASCII codepoints, and representative strings including expansions, combining
marks, titlecase letters, Georgian, and embedded NUL.

The 41 input scalars whose mappings differ between Unicode 17 and Unicode 18 are
excluded based on differences in the two UCD releases, independently of whether
Roe agrees with MRI. The remaining 3,131 inputs yield 53,227 comparisons.

All fields are hexadecimal UTF-8 bytes. The columns are:

1. Input.
2. `downcase`: default, ASCII, Turkic, Lithuanian, fold.
3. `upcase`: default, ASCII, Turkic, Lithuanian.
4. `capitalize`: default, ASCII, Turkic, Lithuanian.
5. `swapcase`: default, ASCII, Turkic, Lithuanian.

Each operation/option pair occupies its own column. The fixture was captured by
calling `String#public_send` with the corresponding operation and option, then
serializing the result with `String#unpack1('H*')`. Empty fields represent empty
strings. Malformed UTF-8 is tested separately because Roe preserves bytes and
MRI's Unicode mapping paths reject invalid input.

When updating the Unicode data or MRI oracle, regenerate the results and review
changes to the excluded repertoire as well as mapping behavior. Do not
regenerate expected results from Roe.
