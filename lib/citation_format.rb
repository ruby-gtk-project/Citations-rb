# frozen_string_literal: true

require_relative 'i18n'

# CitationFormat is the output format a rendered citation is written in.
#
# Ported from upstream's cratebibtex/src/format.rs: the Format enum (Plain,
# Markdown, LaTeX, in that order, because the order is the one the format drop
# down shows and the one the "format-kind" setting stores), its translated
# names, and the regex passes that turn a filled-in template into a citation.
#
class CitationFormat
  ALL = []

  attr_reader :index, :id

  def initialize(index, id)
    @index = index
    @id = id
  end

  PLAIN = new(0, 'Plain')
  MARKDOWN = new(1, 'Markdown')
  LATEX = new(2, 'LaTeX')

  ALL.push(PLAIN, MARKDOWN, LATEX).freeze

  # Format::from(i32): anything out of range is Plain, the default.
  def self.from_index(index)
    case index.to_i
    when 1 then MARKDOWN
    when 2 then LATEX
    else PLAIN
    end
  end

  # Format::from(String), as the "format-kind" setting and the BibTeX files
  # spell the names.
  def self.from_id(id)
    case id.to_s
    when 'Markdown' then MARKDOWN
    when 'Latex', 'LaTeX' then LATEX
    else PLAIN
    end
  end

  def plain? = self == PLAIN
  def markdown? = self == MARKDOWN
  def latex? = self == LATEX

  def to_s = @id

  def translated_name
    if markdown?
      'Markdown'
    elsif latex?
      'LaTeX'
    else
      # NOTE Plain as in Plain Text
      I18n.translate('Plain')
    end
  end

  # The regex passes. They are upstream's, with Rust's `^`/`$` written as
  # `\A`/`\z` (Rust anchors match the whole haystack, never a line) and `\s`
  # written as `[[:space:]]` (Rust's `\s` is Unicode aware, Ruby's is not).
  MARKUP = ['*', '_'].freeze
  PLAIN_RE = /\*\S[^*]+\S\*|\*{2}[^*]+\*{2}|_\S[^_]+\S_/
  ITALIC_RE = /\*[^*]+\*|_\S[^_]+\S_/
  BOLD_RE = /\*{2}(?<b>[^*]+)\*{2}/
  CLEAR_RE = /[[:space:]]*([,.;:])([[:space:]]*[,.;:][[:space:]]*)+|([[:space:]])[\n[:space:]]+/
  CLEAR_END_RE = /[[:space:]]*[,.;:][[:space:]]*\z/
  CLEAR_START_RE = /\A([[:space:]]*[,.][[:space:]]*)+/
  NO_RE = /([.,])[[:space:]]*([Nn]o|[Vv]ol|pp)\.[[:space:]]*[,.][[:space:]]*/
  EMPTY_RE = /("[[:space:]]*"[[:space:]]*|'[[:space:]]*'[[:space:]]*|\([[:space:]]*\)[[:space:]]*|`[[:space:]]*`[[:space:]]*|\{[[:space:]]*\}[[:space:]]*)/
  MARKDOWN_RE = /(\*{2}[[:space:]]*\*{2}|\*[[:space:]]+\*|_[[:space:]]*_)/
  NON_MARKDOWN_RE = /\*[[:space:]]*\*[[:space:]]*|\*{2}[[:space:]]*\*{2}[[:space:]]*|_[[:space:]]*_[[:space:]]*/
  LATEX_RE = /\\emph\{[[:space:]]*\}|\\textbf\{[[:space:]]*\}/

  class << self
    # Drops the markdown emphasis markers, keeping their contents.
    def format_plain(citation)
      citation.gsub(PLAIN_RE) { |match| trim_markup(match) }
    end

    # Turns the markdown emphasis into \textbf and \emph, and prefixes the
    # citation with its \bibitem.
    def format_latex(citation, key)
      formatted = citation.gsub(BOLD_RE, '\\\\textbf{\k<b>}')
      formatted = formatted.gsub(ITALIC_RE) { |match| "\\emph{#{trim_markup(match)}}" }

      "\\bibitem{#{key}} #{formatted}"
    end

    # Removes the punctuation and quoting left behind by tags that the entry
    # does not have.
    def clear_citation(citation)
      formatted = citation.to_s.dup

      # Remove "no.  ," and "no,   ."
      formatted = formatted.gsub(NO_RE, '\1 ')

      # Remove empty quotes, braces, etc
      while EMPTY_RE.match?(formatted)
        formatted = formatted.gsub(EMPTY_RE, '')
      end

      # Flatten consecutive punctuation
      formatted = formatted.gsub(CLEAR_RE, '\1 ')
      # Turn last punctuation into a period
      formatted = formatted.gsub(CLEAR_END_RE, '.')
      # Remove punctuation at the start, e.g. " , some title" into "some title"
      formatted.gsub(CLEAR_START_RE, '')
    end

    def clear_markdown(citation) = citation.gsub(MARKDOWN_RE, '')
    def clear_non_markdown(citation) = citation.gsub(NON_MARKDOWN_RE, '')
    def clear_latex(citation) = citation.gsub(LATEX_RE, '')

    # "x and y and z" becomes "x, y, and z".
    def format_authors(authors)
      names = authors.to_s.gsub(' and ', ' AND ').gsub(' & ', ' AND ').split(' AND ')

      if names.length > 1
        names[-1] = "and #{names.last}"
      end

      names.join(', ')
    end

    private

      def trim_markup(match)
        match.sub(/\A[*_]+/, '').sub(/[*_]+\z/, '')
      end
  end
end
