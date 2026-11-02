# frozen_string_literal: true

require 'gtk4'
require 'strscan'

require_relative 'entry'

# Bib is an open bibliography: the list of entries a .bib file holds, plus
# whether it has unsaved changes.
#
# Ported from upstream's cratebibtex/src/bib.rs, which is a GObject
# implementing gio::ListModel over a vector of entries, and from the
# nom-bibtex crate it parses with. The Ruby bindings cannot implement
# Gio::ListModel from Ruby, so the entries live in a Gio::ListStore — a real
# list model the sidebar's FilterListModel can take — and Bib wraps it with the
# same API upstream exposes: parse, serialize, add_entry, add_entry_at_pos,
# remove_entry, remove_pos, find_pos, key_exists and save_to_file.
#
class Bib
  # @preamble and @comment blocks are skipped, as nom-bibtex keeps them out of
  # the bibliographies it reports.
  SKIPPED_BLOCKS = %w[preamble comment].freeze
  CLOSING = { '{' => '}', '(' => ')' }.freeze

  attr_reader :model

  def initialize
    @model = Gio::ListStore.new(Entry.gtype)
    @variables = {}
    @modified = false
    @on_modified = []
  end

  def modified? = @modified

  # Called whenever the modified flag changes, the way upstream's
  # notify::modified is connected to.
  def on_modified(&block)
    @on_modified << block
  end

  def modified=(value)
    unless value == @modified
      @modified = value
      @on_modified.each { |block| block.call(value) }
    end
  end

  def n_items = @model.n_items
  def item(position) = @model.get_item(position)
  def entries = Array.new(@model.n_items) { |pos| @model.get_item(pos) }

  # Bib::parse: replaces the whole list with the entries the data holds.
  def parse(data)
    @variables = {}
    parsed = parse_blocks(data.to_s)

    @model.n_items.then do |removed|
      @model.splice(0, removed, parsed)
      parsed.each { |entry| watch(entry) }
    end

    self
  end

  # Bib::serialize: the text written back to the .bib file.
  def serialize = entries.map(&:serialize).join("\n\n")

  def add_entry(entry)
    @model.n_items.then do |position|
      watch(entry)
      @model.append(entry)
      self.modified = true

      position
    end
  end

  def add_entry_at_pos(entry, position)
    [position, @model.n_items].min.then do |pos|
      watch(entry)
      @model.insert(pos, entry)
      self.modified = true
    end
  end

  def remove_entry(citation_key)
    find_pos_by_key(citation_key).then do |pos|
      if pos.nil?
        warn "Could not remove entry #{citation_key}"
      else
        remove_pos(pos)
      end
    end
  end

  def remove_pos(position)
    @model.remove(position)
    self.modified = true
  end

  def find_pos(entry) = find_pos_by_key(entry.citation_key)

  def key_exists?(citation_key)
    entries.any? { |entry| entry.citation_key == citation_key }
  end

  # Bib::save_to_file: writes the serialised bibliography and clears the
  # modified flag.
  def save_to_file(file)
    file.replace_contents(
      serialize,
      nil,
      true,
      Gio::FileCreateFlags::NONE,
      nil,
    )
    self.modified = false

    self
  end

  private

    def find_pos_by_key(citation_key)
      entries.index { |entry| entry.citation_key == citation_key }
    end

    # Any property change on an entry makes the bibliography modified, which is
    # what enables "Save" and triggers the unsaved-changes dialogs.
    def watch(entry)
      entry.signal_connect('notify') do
        self.modified = true
      end
    end

    # The parser. nom-bibtex reads a sequence of @-blocks and ignores
    # everything between them, which is how BibTeX files carry loose comments.
    def parse_blocks(data)
      scanner = StringScanner.new(data)
      parsed = []

      until scanner.eos?
        if scanner.skip(/[^@]*@/).nil?
          scanner.terminate
        else
          read_block(scanner).then do |entry|
            unless entry.nil?
              parsed << entry
            end
          end
        end
      end

      parsed
    end

    def read_block(scanner)
      scanner.skip(/[[:space:]]*/)
      type = scanner.scan(/[[:alnum:]_-]+/)
      scanner.skip(/[[:space:]]*/)
      opening = scanner.scan(/[{(]/)

      if type.nil? || opening.nil?
        nil
      else
        read_block_body(scanner, type, opening)
      end
    end

    def read_block_body(scanner, type, opening)
      closing = CLOSING.fetch(opening)

      if SKIPPED_BLOCKS.include?(type.downcase)
        skip_block(scanner, opening, closing)
        nil
      elsif type.downcase == 'string'
        read_variable(scanner, closing)
        nil
      else
        read_entry(scanner, type, closing)
      end
    end

    def skip_block(scanner, opening, closing)
      depth = 1

      until depth.zero? || scanner.eos?
        scanner.skip(/[^#{Regexp.escape(opening)}#{Regexp.escape(closing)}]*/)

        if scanner.scan(/#{Regexp.escape(opening)}/)
          depth += 1
        elsif scanner.scan(/#{Regexp.escape(closing)}/)
          depth -= 1
        else
          scanner.terminate
        end
      end
    end

    # @string{name = "value"}: a variable later entries can refer to.
    def read_variable(scanner, closing)
      scanner.skip(/[[:space:]]*/)
      name = scanner.scan(/[^=[:space:]]+/)
      scanner.skip(/[[:space:]]*=[[:space:]]*/)
      value = read_value(scanner)
      scanner.skip(/[[:space:]]*#{Regexp.escape(closing)}/)

      unless name.nil?
        @variables[name.downcase] = value
      end
    end

    def read_entry(scanner, type, closing)
      scanner.skip(/[[:space:]]*/)
      citation_key = scanner.scan(/[^,#{Regexp.escape(closing)}[:space:]]*/).to_s
      scanner.skip(/[[:space:]]*,?/)

      Entry.from_bibliography(type, citation_key, read_tags(scanner, closing))
    end

    def read_tags(scanner, closing)
      {}.tap do |tags|
        done = false

        until done || scanner.eos?
          scanner.skip(/[[:space:]]*/)

          if scanner.scan(/#{Regexp.escape(closing)}/)
            done = true
          else
            read_tag(scanner, tags, closing)
          end
        end
      end
    end

    def read_tag(scanner, tags, closing)
      name = scanner.scan(/[^=,#{Regexp.escape(closing)}[:space:]]+/)
      scanner.skip(/[[:space:]]*/)

      if name.nil?
        scanner.skip(/./)
      elsif scanner.skip(/=[[:space:]]*/)
        tags[name.downcase] = read_value(scanner)
        scanner.skip(/[[:space:]]*,?/)
      else
        scanner.skip(/[[:space:]]*,?/)
      end
    end

    # A value is one or more atoms joined by "#": a quoted string, a braced
    # string, a number, or the name of an @string variable.
    def read_value(scanner)
      parts = [read_atom(scanner)]

      while scanner.skip(/[[:space:]]*#[[:space:]]*/)
        parts << read_atom(scanner)
      end

      parts.join
    end

    def read_atom(scanner)
      scanner.skip(/[[:space:]]*/)

      if scanner.check(/"/)
        read_quoted(scanner)
      elsif scanner.check(/\{/)
        read_braced(scanner)
      else
        read_bare(scanner)
      end
    end

    def read_quoted(scanner)
      scanner.skip(/"/)
      value = +''
      depth = 0
      done = false

      until done || scanner.eos?
        value << scanner.scan(/[^"{}]*/).to_s

        if scanner.scan(/\{/)
          depth += 1
          value << '{'
        elsif scanner.scan(/\}/)
          depth -= 1
          value << '}'
        elsif depth.positive?
          value << scanner.scan(/./).to_s
        else
          scanner.skip(/"/)
          done = true
        end
      end

      value
    end

    def read_braced(scanner)
      scanner.skip(/\{/)
      value = +''
      depth = 1

      until depth.zero? || scanner.eos?
        value << scanner.scan(/[^{}]*/).to_s

        if scanner.scan(/\{/)
          depth += 1
          value << '{'
        elsif scanner.scan(/\}/)
          depth -= 1
          if depth.positive?
            value << '}'
          end
        else
          scanner.terminate
        end
      end

      value
    end

    # A bare atom is a number, or a variable name to look up.
    def read_bare(scanner)
      text = scanner.scan(/[^,#)}[:space:]]+/).to_s

      @variables.fetch(text.downcase, text)
    end
end
