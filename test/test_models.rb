# frozen_string_literal: true

# Ported from the #[test] functions in upstream's cratebibtex/src/format.rs,
# cratebibtex/src/entry.rs, cratebibtex/src/lib.rs and src/utils.rs.

require_relative 'test_helper'
require 'bib'
require 'citation_format'
require 'settings'
require 'utils'

T = TestHelper
F = CitationFormat

T.test('format::test_clear_citation') do
  T.assert_equal('some, text 2', F.clear_citation('some,  text  2'))
  T.assert_equal('some, text', F.clear_citation('some, , , text'))
  T.assert_equal('some, text', F.clear_citation('some, ,  , ,  , text'))
  T.assert_equal('some, text', F.clear_citation('some, .  . ,  . text'))
  T.assert_equal('some. text', F.clear_citation('some. .  . ,  . , text'))
  T.assert_equal('some. text', F.clear_citation('some..,., ,, . , text'))
  T.assert_equal('some, text', F.clear_citation('some, no. , text'))
  T.assert_equal('some. text', F.clear_citation('some. No. . text'))
  T.assert_equal('some. text', F.clear_citation('some. no. , text'))
  T.assert_equal('some. text', F.clear_citation('some. vol. , text'))
  T.assert_equal('some. text', F.clear_citation('some. Vol. , text'))
  T.assert_equal('some. text', F.clear_citation('some. {} , text'))
  T.assert_equal('some. text', F.clear_citation('some. () , text'))
  T.assert_equal('some.', F.clear_citation('some., ,'))
  T.assert_equal('some.', F.clear_citation(' , .some., ,'))
  T.assert_equal('', F.clear_citation('""'))
  T.assert_equal('some, text', F.clear_citation('some, , "" , ,  , text'))
  T.assert_equal('"some", text', F.clear_citation('"some", , "" , ,  , text'))
end

T.test('format::test_format_authors') do
  T.assert_equal('x, and y', F.format_authors('x and y'))
  T.assert_equal('x, y, and z', F.format_authors('x and y and z'))
  T.assert_equal('x, and y', F.format_authors('x & y'))
  T.assert_equal('x, and y', F.format_authors('x AND y'))
end

T.test('format::test_clear_latex') do
  T.assert_equal('', F.clear_latex('\\textbf{}'))
  T.assert_equal('', F.clear_latex('\\emph{}'))
end

T.test('format::test_clear_markdown') do
  T.assert_equal(
    'asdas asd _asd_ *asd* asdasd **asd** ',
    F.clear_markdown('asdas asd _asd_ *asd* ** asdasd **asd** **** __'),
  )
end

# The bibliography from upstream's cratebibtex/src/lib.rs: a preamble, a
# comment, loose text, two @string variables and one entry that uses them.
BIBFILE_DATA = <<~BIB
  @preamble{
      "A bibtex preamble" # " another test"
  }

  @Comment{
      Here is a comment.
  }

  Another comment!

  @string ( name= "Charles Vandevoorde")
  @string (github = "https://github.com/charlesvdv")

  @misc {my_citation_key,
      AUTHOR= name,
      title = "nom-bibtex",
      note = "Github: " # github
  }
BIB

T.test('cratebibtex::test') do
  bib = Bib.new
  bib.parse(BIBFILE_DATA)

  T.assert_equal(1, bib.n_items)
  T.assert_equal('Charles Vandevoorde', bib.item(0).author)
  T.assert_equal('nom-bibtex', bib.item(0).title)
  T.assert_equal('Github: https://github.com/charlesvdv', bib.item(0).note)
  T.assert_equal('my_citation_key', bib.item(0).citation_key)
  T.assert_equal('misc', bib.item(0).entry_type.id)
end

ENTRY_DATA = <<~BIB
  @misc {my_citation_key,
      AUTHOR= "Some Author",
      title = "Some Title",
      year = "2020",
      volume = "5",
  }
BIB

T.test('entry::format') do
  bib = Bib.new
  bib.parse(ENTRY_DATA)
  entry = bib.item(0)
  plain = CitationFormat::PLAIN
  markdown = CitationFormat::MARKDOWN
  latex = CitationFormat::LATEX

  T.assert_equal(
    'Some Author: "Some Title" (2020)',
    entry.format_citation('%AUTHOR%: "%TITLE%" (%YEAR%)', plain),
  )

  template = '%AUTHOR%: "_%TITLE%_" (**%VOLUME%**) (%YEAR%)'
  T.assert_equal('Some Author: "Some Title" (5) (2020)', entry.format_citation(template, plain))
  T.assert_equal(
    'Some Author: "_Some Title_" (**5**) (2020)',
    entry.format_citation(template, markdown),
  )
  T.assert_equal(
    '\\bibitem{my_citation_key} Some Author: "\\emph{Some Title}" (\\textbf{5}) (2020)',
    entry.format_citation(template, latex),
  )

  T.assert_equal(
    'Some Author, Some Title, 5, 2020',
    entry.format_citation('%AUTHOR%, %TITLE%, %VOLUME%, %JOURNAL%, %YEAR%', plain),
  )
  T.assert_equal(
    'Some Author, Some Title, 5, 2020',
    entry.format_citation('%AUTHOR%, %TITLE%, %VOLUME%, "%JOURNAL%", %YEAR%', plain),
  )
  T.assert_equal(
    'Some Author, Some Title, 5, 2020',
    entry.format_citation('%AUTHOR%, %TITLE%, %VOLUME%, (%JOURNAL%), "%FAKE%", %YEAR%', plain),
  )
  T.assert_equal(
    'Some Author, "Some Title", 5, 2020',
    entry.format_citation(%q(%AUTHOR%, "%TITLE%", %VOLUME% {%JOURNAL%}, '%FAKE%',%YEAR%), plain),
  )
  T.assert_equal(
    '\\bibitem{my_citation_key} Some Author, "\\emph{Some Title}", 5, 2020',
    entry.format_citation(%q(%AUTHOR%, "*%TITLE%*", %VOLUME% *%JOURNAL%*, '%FAKE%',%YEAR%), latex),
  )
  T.assert_equal(
    '\\bibitem{my_citation_key} Some Author, "\\emph{Some Title}", 5, 2020',
    entry.format_citation(%q(%AUTHOR%, "*%TITLE%*", %VOLUME% (*%JOURNAL%*), '%FAKE%',%YEAR%), latex),
  )
end

T.test('entry::serialize') do
  bib = Bib.new
  bib.parse(ENTRY_DATA)

  T.assert_equal(
    "@misc{my_citation_key,\n    author = {Some Author},\n    title = {Some Title}," \
    "\n    year = {2020},\n    volume = {5},\n}",
    bib.item(0).serialize,
  )
end

T.test('bib::list_model') do
  bib = Bib.new
  bib.parse(ENTRY_DATA)
  bib.add_entry(Entry.new('newkey', EntryType::BOOK))

  T.assert_equal(2, bib.n_items)
  T.assert(bib.key_exists?('newkey'))
  T.assert(bib.modified?)
  T.assert_equal(1, bib.find_pos(bib.item(1)))

  bib.remove_entry('newkey')
  T.assert_equal(1, bib.n_items)

  bib.add_entry_at_pos(Entry.new('first', EntryType::BOOK), 0)
  T.assert_equal('first', bib.item(0).citation_key)
end

T.test('bib::round_trips_through_a_file') do
  require 'tmpdir'

  Dir.mktmpdir('citations-bib') do |dir|
    path = File.join(dir, 'round-trip.bib')
    bib = Bib.new
    bib.parse(ENTRY_DATA)
    bib.save_to_file(Gio::File.new_for_path(path))

    T.assert_equal(false, bib.modified?)

    reloaded = Bib.new
    reloaded.parse(File.read(path))
    T.assert_equal(1, reloaded.n_items)
    T.assert_equal('Some Author', reloaded.item(0).author)
    T.assert_equal('5', reloaded.item(0).volume)
  end
end

T.test('entry_type::fields_and_names') do
  T.assert_equal(16, EntryType::ALL.length)
  T.assert_equal('PhD Thesis', EntryType.from_id('PhdThesis').translated_name)
  T.assert_equal('other', EntryType.from_index(99).id)
  T.assert_equal(
    %w[journal volume number pages month note issn eprint],
    EntryType.from_index(1).fields,
  )
  T.assert(EntryType::ARTICLE.uses_abstract?)
  T.assert_equal(false, EntryType::BOOK.uses_abstract?)
end

T.test('format::kinds') do
  T.assert_equal('Plain', CitationFormat.from_index(0).translated_name)
  T.assert_equal('Markdown', CitationFormat.from_index(1).translated_name)
  T.assert_equal('LaTeX', CitationFormat.from_index(2).translated_name)
  T.assert_equal('Plain', CitationFormat.from_index(9).translated_name)
  T.assert_equal('LaTeX', CitationFormat.from_id('Latex').translated_name)
end

T.test('utils::test_generate_key') do
  T.assert_equal('a1123456', Utils.generate_key('a', '1', '1234567'))
  T.assert_equal('a1123456', Utils.generate_key('a, b', '1', '12345678'))
  T.assert_equal('a1123456', Utils.generate_key('a, b, c', '1', '1234569'))
  T.assert_equal('a1123456', Utils.generate_key('A, b, c', '1', '1234569'))
  T.assert_equal('a1z', Utils.generate_key('A, b, c', '1', 'z'))
end

T.test('utils::test_sanitize_key') do
  T.assert_equal('a_b_', Utils.sanitize_key("a b'"))
end

T.test('utils::urls_and_filenames') do
  T.assert_equal('key2020.pdf', Utils.pdf_filename('key2020'))
  T.assert_equal('ab.pdf', Utils.pdf_filename('a/b'))
  T.assert_equal('https://arxiv.org/abs/2101.00001', Utils.eprint_url('2101.00001'))
  T.assert_equal('https://arxiv.org/abs/2101.00001', Utils.eprint_url('arxiv.org/abs/2101.00001'))
  T.assert_equal('https://example.com/x.pdf', Utils.eprint_url('https://example.com/x.pdf'))
  T.assert_equal(nil, Utils.eprint_url(''))
  T.assert_equal('https://doi.org/10.1000/182', Utils.doi_url('10.1000/182'))
  T.assert_equal('https://doi.org/10.1000/182', Utils.doi_url('doi.org/10.1000/182'))
  T.assert_equal(nil, Utils.doi_url(''))
  T.assert_equal('Author', Utils.translated_key('author'))
  T.assert_equal('How Published', Utils.translated_key('howpublished'))
  T.assert_equal('ISBN', Utils.translated_key('isbn'))
  T.assert(Utils.build_google_scholar_url('A B', 'T').start_with?('https://scholar.google.com/scholar?q='))
  T.assert(Utils.build_arxiv_url('A B', 'T').include?('terms-0-term=T'))
end

T.test('settings::defaults_match_the_upstream_schema') do
  require 'tmpdir'

  Dir.mktmpdir('citations-settings') do |dir|
    settings = Settings.new(path: File.join(dir, 'settings.json'))

    T.assert_equal(800, settings['window-width'])
    T.assert_equal(600, settings['window-height'])
    T.assert_equal(false, settings['is-maximized'])
    T.assert_equal([], settings['recent-files'])
    T.assert_equal(0, settings['template-kind'])
    T.assert_equal(0, settings['format-kind'])
    T.assert_equal(
      '%AUTHOR%, *%TITLE%*, %PUBLISHER%, %CITY%, %YEAR%.',
      settings['custom-book-template'],
    )
    T.assert_equal(
      '%AUTHOR%, *%TITLE%*, %JOURNAL% **%VOLUME%** (%YEAR%), no. %NUMBER%, %PAGES%.',
      settings['custom-article-template'],
    )

    changed = []
    settings.connect_changed('recent-files') { |key| changed << key }
    settings.set_strv('recent-files', ['/tmp/a.bib'])

    T.assert_equal(['recent-files'], changed)
    T.assert_equal(
      ['/tmp/a.bib'],
      Settings.new(path: File.join(dir, 'settings.json')).strv('recent-files'),
    )
  end
end

T.report('models')
