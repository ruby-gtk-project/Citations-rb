# frozen_string_literal: true

require 'gtk4'
require 'net/http'
require 'uri'

require_relative 'citation_format'
require_relative 'entry'
require_relative 'i18n'

# Utils holds the helpers the widgets share.
#
# Ported from upstream's src/utils.rs: the translated field labels, the search
# URL builders, the bibliography folder, the PDF file naming, the eprint and
# DOI URL guessing, opening a PDF, fetching BibTeX for a DOI, and the citation
# key generator.
#
module Utils
  SHORT_TITLE_LEN = 6

  # The characters a BibTeX label may not contain, based on
  # https://tex.stackexchange.com/questions/581901/what-is-the-safe-character-set-for-a-bibtex-label
  UNSAFE_KEY_CHARS = [
    ' ', '{', '}', '(', ')', ',', '"', "'", '=', '#', '%', '\\',
  ].freeze

  # The characters a file name may not contain.
  UNSAFE_FILENAME_RE = %r{[/\\?%*:|"<>\x00-\x1f]}

  SCHOLAR_URL = 'https://scholar.google.com/scholar'
  ARXIV_URL = 'https://arxiv.org/search/advanced'
  DOI_RESOLVER = 'https://doi.org'

  # The translated labels below go through the shorthands every other file
  # in the port uses, so that the message extractor sees them.
  extend I18n::Translatable

  module_function

  # The label the editor shows for a BibTeX tag.
  def translated_key(key)
    case key.to_s
    when 'author' then _('Author')
    when 'year' then _('Year')
    when 'title' then _('Title')
    when 'volume' then _('Volume')
    when 'number' then _('Number')
    when 'pages' then _('Pages')
    when 'publisher' then _('Publisher')
    when 'journal' then _('Journal')
    when 'address' then _('Address')
    # TRANSLATORS Method of publication, see
    # https://www.bibtex.com/f/howpublished-field/
    when 'howpublished' then _('How Published')
    when 'note' then _('Note')
    when 'booktitle' then _('Book Title')
    when 'series' then _('Series')
    when 'archiveprefix' then _('Archive Prefix')
    # TRANSLATORS As in digital print
    when 'eprint' then _('ePrint')
    # TRANSLATORS Identifier used by arXiv, see
    # https://arxiv.org/help/hypertex/bibstyles
    when 'primaryclass' then _('Primary Class')
    when 'month' then _('Month')
    # TRANSLATORS As in chief editor
    when 'editor' then _('Editor')
    # TRANSLATORS As in association
    when 'organization' then _('Organization')
    when 'school' then _('School')
    when 'institution' then _('Institution')
    when 'issn' then 'ISSN'
    when 'isbn' then 'ISBN'
    when 'url' then 'url'
    else key.to_s
    end
  end

  def build_google_scholar_url(author, title)
    authors = CitationFormat.format_authors(author)
    query = {
      'q'      => "#{authors} #{title}",
      'scisbd' => '0',
      'num'    => '10',
      'start'  => '0',
      'safe'   => 'active',
      'filter' => '1',
      'as_vis' => '1',
    }

    "#{SCHOLAR_URL}?#{URI.encode_www_form(query)}"
  end

  def build_arxiv_url(author, title)
    authors = CitationFormat.format_authors(author)
    query = {
      'advanced'                          => '',
      'terms-0-operator'                  => 'AND',
      'terms-0-term'                      => title,
      'terms-0-field'                     => 'title',
      'terms-1-operator'                  => 'OR',
      'terms-1-term'                      => authors,
      'terms-1-field'                     => 'author',
      'date-filter_by'                    => 'all_dates',
      'date-year'                         => '',
      'date-from_date'                    => '',
      'date-to_date'                      => '',
      'date-date_type'                    => 'submitted_date',
      'classification-physics_archives'   => 'all',
      'classification-include_cross_list' => 'include',
      'abstracts'                         => 'show',
      'size'                              => '25',
      'order'                             => '-announced_date_first',
    }

    "#{ARXIV_URL}?#{URI.encode_www_form(query)}"
  end

  # The folder PDFs are kept in. `nil` when the Documents directory is not
  # known, which is what makes the "Bibliography folder not set" error show.
  #
  # Upstream notes that it would rather use a user defined directory, which
  # waits on https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/issues/41
  def bibliography_path
    documents_dir.then do |documents|
      if documents.nil?
        nil
      else
        File.join(documents, 'Bibliography')
      end
    end
  end

  def bibliography_path_string
    bibliography_path.then do |path|
      if path.nil?
        '~/Documents/Bibliography'
      else
        path.sub(/\A#{Regexp.escape(Dir.home)}/, '~')
      end
    end
  end

  # XDG's Documents directory, which glib::user_special_dir reads from
  # user-dirs.dirs and the Ruby bindings do not expose.
  def documents_dir
    from_user_dirs.then do |configured|
      if !configured.nil?
        configured
      elsif Dir.exist?(File.join(Dir.home, 'Documents'))
        File.join(Dir.home, 'Documents')
      else
        nil
      end
    end
  end

  def from_user_dirs
    config_home = ENV.fetch('XDG_CONFIG_HOME', File.join(Dir.home, '.config'))
    file = File.join(config_home, 'user-dirs.dirs')

    if File.exist?(file)
      parse_user_dirs(File.read(file))
    else
      nil
    end
  end

  def parse_user_dirs(contents)
    contents.lines.grep(/\AXDG_DOCUMENTS_DIR=/).first.then do |line|
      if line.nil?
        nil
      else
        line.split('=', 2).last.strip.delete('"').sub('$HOME', Dir.home)
      end
    end
  end

  def pdf_filename(citation_key)
    "#{citation_key}.pdf".gsub(UNSAFE_FILENAME_RE, '')
  end

  def pdf_file(citation_key)
    bibliography_path.then do |path|
      if path.nil?
        nil
      else
        Gio::File.new_for_path(File.join(path, pdf_filename(citation_key)))
      end
    end
  end

  def remove_recent_file(settings, path)
    settings.set_strv('recent-files', settings.strv('recent-files').reject { |x| x == path })
  end

  # The absolute URL for an eprint tag, which is usually a bare arXiv id.
  #
  # Upstream assumes arXiv here rather than reading the archive prefix, and
  # this follows it.
  def eprint_url(url)
    text = url.to_s.strip

    if text.empty?
      nil
    elsif text.include?('https://') || text.include?('http://')
      absolute_url(text)
    else
      id = text.sub('arxiv.org/abs/', '').sub('arxiv.org/', '')
      absolute_url("https://arxiv.org/abs/#{id}")
    end
  end

  def doi_url(url)
    text = url.to_s.strip

    if text.empty?
      nil
    elsif text.include?('https://') || text.include?('http://')
      absolute_url(text)
    else
      absolute_url("#{DOI_RESOLVER}/#{text.sub('doi.org/', '')}")
    end
  end

  # url::Url::parse, which rejects anything that is not an absolute URL.
  def absolute_url(text)
    URI.parse(text).then do |uri|
      if uri.absolute? && !uri.host.to_s.empty?
        uri.to_s
      else
        nil
      end
    end
  rescue URI::InvalidURIError
    nil
  end

  def has_pdf?(citation_key)
    pdf_file(citation_key).then do |pdf|
      if pdf.nil?
        false
      else
        File.exist?(pdf.path)
      end
    end
  end

  # Hands the PDF to the system's document viewer.
  def open_pdf(citation_key, window)
    pdf_file(citation_key).then do |pdf|
      if pdf.nil?
        raise 'There is no bibliography path'
      end

      Gtk::FileLauncher.new(pdf).launch(window)
    end
  end

  # Fetches the BibTeX for a DOI and gives it the key Citations generates,
  # which is what upstream's doi2bib call plus sanitize_key does.
  def bib_from_doi(doi)
    doi_str = doi.to_s.sub('https://', '').sub('http://', '').sub('doi.org/', '').strip
    data = http_get("#{DOI_RESOLVER}/#{doi_str}", 'application/x-bibtex')

    Entry.from_bibtex(data.force_encoding(Encoding::UTF_8)).then do |entry|
      entry.citation_key = sanitize_key(generate_key(entry.author, entry.year, entry.title))

      entry.serialize
    end
  end

  # "lastname" + year + the first six characters of the title.
  def generate_key(authors, year, title)
    last_name = authors.to_s.split(/[ ,]/).first.to_s.downcase
    short_title = title.to_s[0, SHORT_TITLE_LEN].to_s

    "#{last_name}#{year}#{short_title}"
  end

  def sanitize_key(key)
    key.to_s.each_char.map { |char| safe_key_char(char) }.join
  end

  def safe_key_char(char)
    if UNSAFE_KEY_CHARS.include?(char)
      '_'
    else
      char
    end
  end

  # The crate's utils::get: the bytes behind a URL, following redirects.
  def get(url) = http_get(url, nil)

  def http_get(url, accept, redirects = 5)
    uri = URI.parse(url)
    request = Net::HTTP::Get.new(uri)
    unless accept.nil?
      request['Accept'] = accept
    end

    Net::HTTP.start(uri.host, uri.port, :use_ssl => uri.scheme == 'https') do |http|
      http.request(request)
    end.then { |response| handle_response(response, accept, redirects) }
  end

  def handle_response(response, accept, redirects)
    case response
    when Net::HTTPSuccess
      response.body
    when Net::HTTPRedirection
      if redirects.zero?
        raise 'Too many redirects'
      end

      http_get(response['location'], accept, redirects - 1)
    else
      raise "Request failed: #{response.code} #{response.message}"
    end
  end
end
