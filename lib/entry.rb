# frozen_string_literal: true

require 'gtk4'

require_relative 'citation_format'
require_relative 'entry_type'
require_relative 'texer'

# Entry is one citation: its type, its citation key and its BibTeX tags.
#
# Ported from upstream's cratebibtex/src/entry.rs. Upstream is a GObject whose
# named properties all read and write one `tags` map, so that a widget can bind
# to "author" or "title" and the change lands in the map the serialiser walks.
# This is the same object: a GLib::Object with the same property names, each
# one backed by the same tag map, each setter notifying only when the value
# really changed.
#
# `search-text` is the port's own addition. Upstream filters the sidebar with a
# gtk::ClosureExpression over four properties; the Ruby bindings have no usable
# closure expression, so the four are concatenated into one notifying property
# that a Gtk::PropertyExpression can watch instead.
#
class Entry < GLib::Object
  # The string properties, in upstream's order. The key is the property name,
  # the value the BibTeX tag it reads and writes — they differ only for
  # "x-abstract", which cannot be called "abstract" in Rust.
  STRING_PROPERTIES = {
    'notes'         => 'notes',
    'x-abstract'    => 'abstract',
    'author'        => 'author',
    'year'          => 'year',
    'title'         => 'title',
    'volume'        => 'volume',
    'number'        => 'number',
    'pages'         => 'pages',
    'publisher'     => 'publisher',
    'journal'       => 'journal',
    'address'       => 'address',
    'howpublished'  => 'howpublished',
    'note'          => 'note',
    'booktitle'     => 'booktitle',
    'series'        => 'series',
    'archiveprefix' => 'archiveprefix',
    'eprint'        => 'eprint',
    'primaryclass'  => 'primaryclass',
    'month'         => 'month',
    'editor'        => 'editor',
    'organization'  => 'organization',
    'school'        => 'school',
    'institution'   => 'institution',
    'doi'           => 'doi',
    'url'           => 'url',
    'issn'          => 'issn',
    'isbn'          => 'isbn',
  }.freeze

  # The tags that make up "search-text", as upstream's search expression
  # concatenates them.
  SEARCH_TAGS = %w[author title year].freeze

  TAG_RE = /%(?<tag>[A-Z]+)%/

  type_register

  install_property(
    GLib::Param::String.new(
      'citation-key',
      'citation-key',
      'The identifier the bibliography refers to this entry by',
      '',
      GLib::Param::READWRITE,
    ),
  )

  install_property(
    GLib::Param::String.new(
      'other-entry-type',
      'other-entry-type',
      'The BibTeX type name of an entry whose type Citations does not know',
      '',
      GLib::Param::READWRITE,
    ),
  )

  # Upstream's "entry-type" is an enum property. GLib enum types cannot be
  # registered from Ruby, so the port's property carries the discriminant —
  # which is what the citation type drop down reads and writes anyway — while
  # `entry_type` stays the EntryType itself.
  install_property(
    GLib::Param::Int.new(
      'entry-type-index',
      'entry-type-index',
      'The kind of source this entry points at',
      0,
      EntryType::ALL.length - 1,
      EntryType::DEFAULT.index,
      GLib::Param::READWRITE,
    ),
  )

  install_property(
    GLib::Param::String.new(
      'search-text',
      'search-text',
      'Author, title, year and citation key, for the sidebar filter',
      '',
      GLib::Param::READABLE,
    ),
  )

  STRING_PROPERTIES.each_key do |name|
    install_property(
      GLib::Param::String.new(
        name,
        name,
        "The entry's #{STRING_PROPERTIES[name]} tag",
        '',
        GLib::Param::READWRITE,
      ),
    )
  end

  def initialize(citation_key = '', entry_type = EntryType::DEFAULT)
    super()
    @tags = {}
    @citation_key = citation_key.to_s
    @entry_type = entry_type
    @other_entry_type = nil
  end

  # Entry::new(citation_key, type_).
  def self.create(citation_key, entry_type) = new(citation_key, entry_type)

  # Entry::from_bibtex: parses one entry out of a BibTeX snippet. Raises when
  # the snippet holds none, as upstream bails on "Invalid BibTeX data".
  def self.from_bibtex(data)
    require_relative 'bib'

    Bib.new.then do |bib|
      bib.parse(data)
      if bib.n_items.zero?
        raise ArgumentError, 'Invalid BibTeX data'
      end

      bib.item(0)
    end
  end

  # Builds an entry from the type, key and tags a parsed bibliography holds.
  def self.from_bibliography(entry_type_name, citation_key, tags)
    entry_type = EntryType.from_id(entry_type_name)

    new(citation_key, entry_type).tap do |entry|
      entry.replace_tags(tags)
      if entry_type.other?
        entry.other_entry_type = entry_type_name
      end
    end
  end

  attr_reader :citation_key, :other_entry_type

  def entry_type = @entry_type

  # The int the "entry-type" property and the citation type drop down use.
  def entry_type_index = @entry_type.index

  def citation_key=(value)
    value.to_s.then do |key|
      unless key == @citation_key
        @citation_key = key
        notify('citation-key')
        notify('search-text')
      end
    end
  end

  def entry_type=(value)
    coerce_entry_type(value).then do |type|
      unless type == @entry_type
        @entry_type = type
        notify('entry-type-index')
      end
    end
  end

  def entry_type_index=(value)
    self.entry_type = EntryType.from_index(value)
  end

  def other_entry_type=(value)
    @other_entry_type = value.to_s
  end

  def search_text
    SEARCH_TAGS.map { |tag| find_tag(tag).to_s }.push(@citation_key).join(' ')
  end

  # The tag map, as upstream's Entry::tags.
  def tags = @tags.dup

  def find_tag(tag) = @tags[tag.to_s]

  def insert_tag(tag, value)
    @tags[tag.to_s] = value.to_s
  end

  def replace_tags(tags)
    @tags = {}
    tags.each { |tag, value| @tags[tag.to_s.downcase] = value.to_s }
    notify('search-text')
  end

  def has_property?(name) = property_names.include?(name.to_s)

  def property_names
    ['citation-key', 'other-entry-type', 'entry-type-index', 'search-text'] +
      STRING_PROPERTIES.keys
  end

  def fields = @entry_type.fields

  # Reads and writes for every string property, each one going through the tag
  # map and notifying only on a real change — upstream's `explicit_notify`.
  STRING_PROPERTIES.each do |name, tag|
    reader = name.tr('-', '_')

    define_method(reader) { find_tag(tag).to_s }

    define_method(:"#{reader}=") do |value|
      value.to_s.then do |text|
        unless find_tag(tag) == text
          insert_tag(tag, text)
          notify(name)
          if SEARCH_TAGS.include?(tag)
            notify('search-text')
          end
        end
      end
    end
  end

  # Entry::format_citation: fills a template in, then cleans up after the tags
  # the entry does not have.
  def format_citation(template, format)
    formatted = initial_formatting(template, format)
    authors = CitationFormat.format_authors(formatted_author(format))
    formatted = formatted.gsub('%AUTHOR%') { authors }
    formatted = formatted.gsub(TAG_RE) { formatted_tag(Regexp.last_match[:tag].downcase, format) }

    clear(formatted, format)
  end

  # Entry::serialize: the BibTeX text for this entry.
  def serialize
    lines = @tags.map { |tag, value| "    #{tag} = {#{value}}," }

    "@#{serialized_type}{#{@citation_key},\n#{lines.join("\n")}\n}"
  end

  private

    def serialized_type
      if @entry_type.other?
        @other_entry_type || 'other'
      else
        @entry_type.id
      end
    end

    def coerce_entry_type(value)
      case value
      when EntryType then value
      when Integer then EntryType.from_index(value)
      else EntryType.from_id(value)
      end
    end

    def initial_formatting(template, format)
      if format.plain?
        CitationFormat.format_plain(template)
      elsif format.latex?
        CitationFormat.format_latex(template, @citation_key)
      else
        template.dup
      end
    end

    # LaTeX output keeps the LaTeX markup; the other formats resolve it.
    def formatted_author(format)
      if format.latex?
        author
      else
        Texer.texer(author)
      end
    end

    def formatted_tag(tag, format)
      value = find_tag(tag).to_s

      if format.latex? || tag != 'title'
        value
      else
        Texer.texer(value)
      end
    end

    def clear(formatted, format)
      if format.markdown?
        CitationFormat.clear_citation(CitationFormat.clear_markdown(formatted))
      elsif format.latex?
        CitationFormat.clear_citation(
          CitationFormat.clear_non_markdown(CitationFormat.clear_latex(formatted)),
        )
      else
        CitationFormat.clear_citation(CitationFormat.clear_non_markdown(formatted))
      end
    end
end
