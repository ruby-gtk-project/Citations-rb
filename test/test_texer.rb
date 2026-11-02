# frozen_string_literal: true

# Ported from upstream's texer/src/tests.rs, test for test.

require_relative 'test_helper'
require 'texer'

T = TestHelper
X = Texer

T.test('parse_method_name') do
  T.assert_equal(["'", 'a'], [X.parse_nonalpha_method("\\'a")[1], X.parse_nonalpha_method("\\'a")[2]])
end

T.test('parse_method_with_arg') do
  T.assert_equal(['hello', 'some', 'f'], X.parse_method_with_arg('\\some{f}hello'))
  T.assert_equal(['foo', 'textbf', 'hello'], X.parse_method_with_arg('\\textbf{hello}foo'))
  T.assert_equal(['', 'textbf', '\\textbf{hello}'], X.parse_method_with_arg('\\textbf{\\textbf{hello}}'))
  T.assert_equal(['test', 'textbf', nil], X.parse_method_with_arg('\\textbf{}test'))
end

T.test('parse_method_without_arg') do
  T.assert_equal([' foo', 'textbf', nil], X.parse_method_without_arg('\\textbf foo'))
  T.assert_equal(['', 'o', nil], X.parse_method_without_arg('\\o'))
  T.assert_equal([' ', 'o', nil], X.parse_method_without_arg('\\o '))
end

T.test('parse_method') do
  T.assert_equal(['hello', 'foo', 'bar'], X.parse_method('\\foo{bar}hello'))
  T.assert_equal(['hello', 'foo', nil], X.parse_method('\\foo{}hello'))
  T.assert_equal(['', "'", nil], X.parse_method("\\'"))
  T.assert_equal(['', "'", 'u'], X.parse_method("\\'u"))
  T.assert_equal(['', 'o', nil], X.parse_method('\\o'))
end

T.test('parser') do
  T.assert_equal(['', 'hello world'], X.parser('hello world'))
  T.assert_equal(['', 'ø'], X.parser('\\o'))
  T.assert_equal(['', 'ø'], X.parser('\\o{}'))
  T.assert_equal(['c', 'ø'], X.parser('\\o{}c'))
end

T.test('test_35') do
  T.assert_equal("Cie\u0302ncias", X.texer('Ci\\^encias'))
  T.assert_equal('Hello World', X.texer('{{Hello World}}'))
  T.assert_equal("e\u0302", X.texer('\\^e'))
  T.assert_equal("e\u0302n", X.texer('\\^en'))
  T.assert_equal("Cie\u0302ncias Biolo\u0301gicas", X.texer("{Ci\\^encias Biol\\'ogicas}"))
  T.assert_equal("Cie\u0302ncias Biolo\u0301gicas", X.texer("{{Ci\\^encias Biol\\'ogicas}}"))

  long = "Contexto Da P\\'os-Gradua\\c{c}\\~ao Stricto Sensu No \\^Ambito Da " \
         "Forma\\c{c}\\~ao Pedag\\'ogica e a Forma\\c{c}\\~ao Docente Em " \
         "{{Ci\\^encias Biol\\'ogicas}}: o Que Revela a Produ\\c{c}\\~ao Acad\\^emica"
  expected = "Contexto Da Po\u0301s-Graduac\u0327a\u0303o Stricto Sensu No A\u0302mbito Da " \
             "Formac\u0327a\u0303o Pedago\u0301gica e a Formac\u0327a\u0303o Docente Em " \
             "Cie\u0302ncias Biolo\u0301gicas: o Que Revela a Produc\u0327a\u0303o Acade\u0302mica"
  T.assert_equal(expected, X.texer(long))
end

T.test('test_46') do
  T.assert_equal("u\u0308", X.texer("\\\"u"))
  T.assert_equal("u\u0308", X.texer("\\\"{u}"))
  T.assert_equal("Schu\u0308tt", X.texer("Sch\\\"{u}tt"))
end

T.test('delimited_by_brackets') do
  T.assert_equal(['', 'abc'], X.delimited_by_brackets('{abc}'))
  T.assert_equal(['', '\\foo{bar}'], X.delimited_by_brackets('{\\foo{bar}}'))
end

T.test('texer') do
  T.assert_equal('hello world', X.texer('hello world'))
  T.assert_equal('hello wørld', X.texer('hello w\\o{}rld'))
  T.assert_equal('hello wø', X.texer('hello w\\o{}'))
  T.assert_equal('hello wø', X.texer('hello w\\o'))
  T.assert_equal('hello wørld asdf', X.texer('hello w\\o{}rld asdf'))
  T.assert_equal('hello world', X.texer('hello w{o}rld'))
  T.assert_equal('hello world', X.texer('hello {}{}wo{}rld'))
  T.assert_equal('ab', X.texer('{ab}'))
  T.assert_equal('foo bar', X.texer('{foo bar}'))
  T.assert_equal("u\u0301", X.texer("\\'u"))
  T.assert_equal("c\u0327", X.texer('\\c{c}'))
  T.assert_equal("c\u030C", X.texer('\\v{c}'))
  T.assert_equal('ø', X.texer('{\\o}'))
  T.assert_equal('øa', X.texer('{\\o}a'))
  T.assert_equal("o\u0308", X.texer('{\\"o}'))
  T.assert_equal('a some', X.texer('a {some}'))
  T.assert_equal('a some', X.texer('a {\\some}'))
  T.assert_equal("e\u0302", X.texer('\\^e'))
  T.assert_equal('a b', X.texer('a  b'))
  T.assert_equal(' ', X.texer("\n              "))
  T.assert_equal('a b', X.texer('{a  b}'))
  T.assert_equal('a so\\ ^\\ast asda \\me', X.texer('a {$so\\ ^\\ast asda \\me$}'))
  T.assert_equal('ab', X.texer('$ab$'))
  T.assert_equal('a some', X.texer('a {$some$}'))
end

T.test('multispacer') do
  T.assert_equal(['a', ' '], X.multispacer("\n a"))
  T.assert_equal(['a', ' '], X.multispacer("\r  \n\r \n\na"))
  T.assert_equal(['', ' '], X.multispacer("\n  "))
  T.assert_equal(nil, X.multispacer('a'))
end

T.test('till_symbols_no_ws_once') do
  T.assert_equal(['', 'hello'], X.till_symbols_no_ws_once('hello'))
  T.assert_equal(['hello', ' '], X.till_symbols_no_ws_once(' hello'))
  T.assert_equal(['hello', ' '], X.till_symbols_no_ws_once('  hello'))
  T.assert_equal(['hello', ' '], X.till_symbols_no_ws_once(" \n hello"))
  T.assert_equal(['', 'a'], X.till_symbols_no_ws_once('a'))
  T.assert_equal(['  b', 'a'], X.till_symbols_no_ws_once('a  b'))
  T.assert_equal(['  ', 'a'], X.till_symbols_no_ws_once('a  '))
  T.assert_equal(['b', ' '], X.till_symbols_no_ws_once('  b'))
  T.assert_equal([' b', 'a'], X.till_symbols_no_ws_once('a b'))
  T.assert_equal(['b', ' '], X.till_symbols_no_ws_once(' b'))
end

T.test('till_symbols_no_ws') do
  T.assert_equal(['', ' hello'], X.till_symbols_no_ws(" \n hello"))
  T.assert_equal(['', 'a b'], X.till_symbols_no_ws('a  b'))
  T.assert_equal(['', 'hello'], X.till_symbols_no_ws('hello'))
  T.assert_equal(['', 'hello world'], X.till_symbols_no_ws('hello world'))
  T.assert_equal(['', 'hello world'], X.till_symbols_no_ws('hello  world'))
end

T.test('tag_consecutive_spaces') do
  T.assert_equal(['', ''], X.tag_consecutive_spaces(''))
  T.assert_equal(['', 'a b'], X.tag_consecutive_spaces('a b'))
  T.assert_equal(['', 'a b c'], X.tag_consecutive_spaces('a b c'))
  T.assert_equal(['', 'hello world'], X.tag_consecutive_spaces('hello world'))
  T.assert_equal(['  c', 'a'], X.tag_consecutive_spaces('a  c'))
  T.assert_equal(['  c', 'aa'], X.tag_consecutive_spaces('aa  c'))
  T.assert_equal(['  c  d', 'aa'], X.tag_consecutive_spaces('aa  c  d'))
  T.assert_equal(['  ', ''], X.tag_consecutive_spaces('  '))
  T.assert_equal([" \n ", ''], X.tag_consecutive_spaces(" \n "))
end

# NOTE U+2009 is THIN SPACE.
T.test('test_59') do
  T.assert_equal(["\u2009 E", 's:'], X.tag_consecutive_spaces("s:\u2009 E"))
  T.assert_equal(['a', ' '], X.multispacer("\u2009 a"))
  T.assert_equal(['', 'Models: Evidence'], X.till_symbols_no_ws("Models:\u2009 Evidence"))
end

T.report('texer')
