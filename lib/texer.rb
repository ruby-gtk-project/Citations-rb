# frozen_string_literal: true

# Texer turns LaTeX markup into plain text.
#
# Ported from upstream's texer/src/lib.rs and texer/src/utils.rs, a nom parser
# crate. Every author and title shown in the sidebar, and every search URL
# built for Google Scholar or arXiv, passes through `Texer.texer`.
#
# The nom combinators are kept recognisable: each one becomes a method that
# takes the remaining input and answers either `nil` (the parse failed, which
# is nom's `Err`) or a two element array of the unconsumed input and the value
# it produced (nom's `Ok((input, output))`).
#
module Texer
  # \o, \O, \l and \L are stand-alone symbols rather than accents.
  SYMBOLS = {
    'o' => 'ø',
    'O' => 'Ø',
    'l' => 'ł',
    'L' => 'Ł',
  }.freeze

  # The accent methods, mapped to the combining mark that is appended to the
  # character they accent.
  COMBINING = {
    "'" => "́",  # COMBINING ACUTE ACCENT
    '`' => "̀",  # COMBINING GRAVE
    '~' => "̃",  # COMBINING TILDE
    '^' => "̂",  # COMBINING CIRCUMFLEX ACCENT
    '"' => "̈",  # COMBINING DIAERESIS
    '=' => "̄",  # COMBINING MACRON
    'c' => "̧",  # COMBINING CEDILLA
    'u' => "̆",  # COMBINING BREVE
    'v' => "̌",  # COMBINING CARON
    'k' => "̨",  # COMBINING OGONEK
  }.freeze

  # THIN SPACE, which BibTeX files use between a colon and the subtitle.
  THIN_SPACE = " "

  WHITESPACE = /\A[[:space:]]/
  SYMBOL_START = /\A[^{\\$[:space:]]+/
  METHOD_NAME = /\A\\([[:alpha:]]+)/
  MATH = /\A\$([^$]*)\$/

  module_function

  # The crate's entry point: parses the whole string, and falls back to the
  # input unchanged when nothing was produced.
  def texer(og_input)
    result = +''
    rest = og_input
    first = true
    done = false

    until done
      parsed = nil
      if rest.empty?
        done = true
      else
        parsed = parser(rest)
      end

      if done
        nil
      elsif parsed.nil?
        # nom's Err branch: the rest of the input is taken verbatim.
        result << rest
        done = true
      elsif first && parsed.first.empty?
        result = parsed.last.dup
        done = true
      else
        first = false
        result << parsed.last
        rest = parsed.first
      end
    end

    if result.empty?
      og_input
    else
      result
    end
  end

  def parser(input)
    if input.start_with?('\\')
      parse_escape(input)
    elsif input.start_with?('$')
      parse_math(input)
    elsif input.start_with?('{')
      parse_group(input)
    else
      till_symbols_no_ws(input)
    end
  end

  # `\method`, `\method{argument}` or `\'a`.
  def parse_escape(input)
    parsed = parse_method(input)

    if parsed.nil?
      nil
    else
      rest, method, argument = parsed
      [rest, expand_method(method, argument)]
    end
  end

  def expand_method(method, argument)
    if argument.nil? && SYMBOLS.key?(method)
      SYMBOLS[method]
    elsif argument.nil?
      method
    elsif argument.length == 1 && method.length == 1
      combine_unicode(method, argument)
    else
      "#{method}#{argument}"
    end
  end

  # Appends the combining mark for `method` to `data`, as texer's
  # utils::combine_unicode does.
  def combine_unicode(method, data)
    if COMBINING.key?(method)
      "#{data}#{COMBINING[method]}"
    else
      data
    end
  end

  def parse_method(input)
    parse_method_with_arg(input) ||
      parse_method_without_arg(input) ||
      parse_nonalpha_method(input)
  end

  def parse_method_with_arg(input)
    match = METHOD_NAME.match(input)

    if match.nil?
      nil
    else
      parse_method_argument(match[1], match.post_match)
    end
  end

  def parse_method_argument(method, input)
    delimited = delimited_by_brackets(input)

    if delimited.nil?
      nil
    elsif delimited.last.empty?
      [delimited.first, method, nil]
    else
      [delimited.first, method, delimited.last]
    end
  end

  def parse_method_without_arg(input)
    match = METHOD_NAME.match(input)

    if match.nil?
      nil
    else
      [match.post_match, match[1], nil]
    end
  end

  # `\'`, `\'a` and `\'{a}`: a one character method name that is not a letter,
  # whose argument is a single character.
  def parse_nonalpha_method(input)
    body = input.delete_prefix('\\')

    if input == body || body.empty?
      nil
    else
      parse_nonalpha_argument(body[0], body[1..])
    end
  end

  def parse_nonalpha_argument(method, rest)
    if WHITESPACE.match?(rest)
      [rest, method, nil]
    elsif /\A[[:alpha:]]/.match?(rest)
      [rest[1..], method, rest[0]]
    elsif rest.start_with?('{')
      parse_braced_character(method, rest)
    else
      [rest, method, nil]
    end
  end

  def parse_braced_character(method, rest)
    if rest.length >= 3 && rest[2] == '}'
      [rest[3..], method, rest[1]]
    else
      nil
    end
  end

  # `$...$`, which BibTeX uses for inline maths. Its contents are taken
  # verbatim.
  def parse_math(input)
    match = MATH.match(input)

    if match.nil?
      nil
    else
      [match.post_match, match[1]]
    end
  end

  # `{...}`: the braces are dropped and their contents parsed in turn.
  def parse_group(input)
    delimited = delimited_by_brackets(input)

    if delimited.nil?
      nil
    else
      parse_group_body(delimited.first, delimited.last)
    end
  end

  def parse_group_body(rest, body)
    acc = +''
    remainder = body
    failed = false

    until remainder.empty? || failed
      parsed = parser(remainder)

      if parsed.nil?
        failed = true
      else
        remainder = parsed.first
        acc << parsed.last
      end
    end

    if failed
      nil
    else
      [rest, acc]
    end
  end

  def delimited_by_brackets(input)
    if input.start_with?('{')
      take_until_unbalanced(input[1..]).then do |inner|
        if inner.nil?
          nil
        else
          [input[(inner.length + 2)..], inner]
        end
      end
    else
      nil
    end
  end

  # Everything up to the first unmatched closing brace, which is not consumed.
  # `nil` when the braces never close.
  def take_until_unbalanced(input)
    depth = 0
    index = nil

    input.each_char.with_index do |char, pos|
      if !index.nil?
        nil
      elsif char == '{'
        depth += 1
      elsif char == '}' && depth.zero?
        index = pos
      elsif char == '}'
        depth -= 1
      end
    end

    if index.nil?
      nil
    else
      input[0, index]
    end
  end

  def till_symbol(input)
    match = /\A[^{\\$]*/.match(input)

    [match.post_match, match[0]]
  end

  def till_symbol1(input)
    match = SYMBOL_START.match(input)

    if match.nil?
      nil
    else
      [match.post_match, match[0]]
    end
  end

  # Collapses a run of whitespace into a single space.
  def multispacer(input)
    if WHITESPACE.match?(input)
      [input.sub(/\A[[:space:]]+/, ''), ' ']
    else
      nil
    end
  end

  def weird_whitespaces(input)
    if input.start_with?(THIN_SPACE)
      [input[1..], ' ']
    else
      nil
    end
  end

  def till_symbols_no_ws_once(input)
    multispacer(input) || till_symbol1(input) || weird_whitespaces(input)
  end

  def till_symbols_no_ws(input)
    rest, data = tag_consecutive_spaces(input)

    if rest.empty?
      till_symbol(data)
    else
      fold_till_symbols(rest, data)
    end
  end

  def fold_till_symbols(rest, data)
    acc = data.dup
    remainder = rest
    done = false

    until done
      parsed = till_symbols_no_ws_once(remainder)

      if parsed.nil?
        done = true
      else
        remainder = parsed.first
        acc << parsed.last
      end
    end

    [remainder, acc]
  end

  # Parses until two or more consecutive whitespace characters are found.
  def tag_consecutive_spaces(input)
    chars = input.each_char.to_a
    index = nil

    chars.each_with_index do |char, pos|
      next_char = chars[pos + 1]

      if !index.nil? || next_char.nil?
        nil
      elsif WHITESPACE.match?(char) && WHITESPACE.match?(next_char)
        index = pos
      end
    end

    if index.nil?
      ['', input]
    else
      [input[index..], input[0, index]]
    end
  end
end
