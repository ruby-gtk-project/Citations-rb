# frozen_string_literal: true

require_relative 'i18n'

# EntryType is the kind of source a citation points at.
#
# Ported from upstream's cratebibtex/src/entry_type.rs: the sixteen BibTeX
# entry types in the enum's own order (Other first, because it is the default
# and the discriminant the "Other…" row of the citation type drop down uses),
# their BibTeX names, their translated names, whether they carry an abstract
# rather than notes, and the extra fields each one adds to the editor.
#
class EntryType
  ALL = []

  attr_reader :index, :id, :fields

  def initialize(index, id, fields, uses_abstract)
    @index = index
    @id = id
    @fields = fields.freeze
    @uses_abstract = uses_abstract
  end

  # NOTE The translated names are built on demand rather than stored, so that
  # a language change is picked up without rebuilding the enum.
  def translated_name
    case @id
    when 'article' then I18n.translate('Article')
    when 'book' then I18n.translate('Book')
    when 'misc' then I18n.translate('Misc')
    # NOTE As in conference proceedings, see
    # https://en.wikipedia.org/wiki/Conference_proceeding
    when 'inproceedings' then I18n.translate('In Proceedings')
    when 'unpublished' then I18n.translate('Unpublished')
    # NOTE As in online source
    when 'online' then I18n.translate('Online')
    when 'booklet' then I18n.translate('Booklet')
    when 'conference' then I18n.translate('Conference')
    when 'inbook' then I18n.translate('In Book')
    when 'incollection' then I18n.translate('In Collection')
    # NOTE As in instruction manual
    when 'manual' then I18n.translate('Manual')
    when 'masterthesis' then I18n.translate("Master's Thesis")
    when 'phdthesis' then I18n.translate('PhD Thesis')
    # NOTE As in conference proceedings, see
    # https://en.wikipedia.org/wiki/Conference_proceeding
    when 'proceedings' then I18n.translate('Proceedings')
    when 'techreport' then I18n.translate('Tech Report')
    # NOTE As in other kind of source
    else I18n.translate('Other')
    end
  end

  def uses_abstract? = @uses_abstract
  def other? = @id == 'other'
  def to_s = @id

  OTHER = new(
    0,
    'other',
    [],
    false,
  )
  ARTICLE = new(
    1,
    'article',
    %w[journal volume number pages month note issn eprint],
    true,
  )
  BOOK = new(
    2,
    'book',
    %w[publisher address editor volume number pages month note isbn eprint],
    false,
  )
  MISC = new(
    3,
    'misc',
    %w[howpublished note eprint],
    false,
  )
  IN_PROCEEDINGS = new(
    4,
    'inproceedings',
    %w[booktitle series pages publisher address volume number isbn],
    true,
  )
  UNPUBLISHED = new(
    5,
    'unpublished',
    [],
    true,
  )
  ONLINE = new(
    6,
    'online',
    %w[url archiveprefix eprint primaryclass],
    true,
  )
  BOOKLET = new(
    7,
    'booklet',
    %w[howpublished month],
    false,
  )
  CONFERENCE = new(
    8,
    'conference',
    %w[booktitle series pages publisher address],
    false,
  )
  IN_BOOK = new(
    9,
    'inbook',
    %w[booktitle address pages],
    false,
  )
  IN_COLLECTION = new(
    10,
    'incollection',
    %w[editor booktitle publisher address pages],
    true,
  )
  MANUAL = new(
    11,
    'manual',
    %w[organization address],
    false,
  )
  MASTER_THESIS = new(
    12,
    'masterthesis',
    %w[school address month],
    true,
  )
  PHD_THESIS = new(
    13,
    'phdthesis',
    %w[school month address],
    true,
  )
  PROCEEDINGS = new(
    14,
    'proceedings',
    %w[editor series volume publisher address],
    true,
  )
  TECH_REPORT = new(
    15,
    'techreport',
    %w[institution address number month],
    false,
  )

  ALL.push(
    OTHER,
    ARTICLE,
    BOOK,
    MISC,
    IN_PROCEEDINGS,
    UNPUBLISHED,
    ONLINE,
    BOOKLET,
    CONFERENCE,
    IN_BOOK,
    IN_COLLECTION,
    MANUAL,
    MASTER_THESIS,
    PHD_THESIS,
    PROCEEDINGS,
    TECH_REPORT,
  ).freeze

  DEFAULT = OTHER

  # EntryType::from(i32): an unknown discriminant is Other.
  def self.from_index(index)
    ALL.find { |type| type.index == index.to_i } || DEFAULT
  end

  # EntryType::from(&str): the BibTeX type name, case insensitively.
  def self.from_id(id)
    ALL.find { |type| type.id == id.to_s.downcase } || DEFAULT
  end
end
