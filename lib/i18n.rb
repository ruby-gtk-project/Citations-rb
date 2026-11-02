# frozen_string_literal: true

require 'gettext'

# I18n binds the gettext domain the port shares with upstream.
#
# Ported from the gettext bootstrap in upstream's src/main.rs, which calls
# setlocale, bindtextdomain and textdomain for the "citations" domain before
# anything builds a widget, and from po/POTFILES.in, which lists the files the
# catalogues are extracted from.
#
# Upstream's Rust code calls gettextrs::gettext directly; here the same calls
# are `_('…')` and `p_('context', '…')`, the names the gettext gem's message
# extractor recognises, which keeps the msgids identical to upstream's.
#
module I18n
  DOMAIN = 'citations'

  # The catalogues live next to the sources in a checkout, exactly as the
  # compiled ones live under <prefix>/share/locale in an installed copy.
  LOCALE_PATH = File.expand_path('../po', __dir__)

  # GetText is designed to be mixed into a class, so the module level
  # translations go through one instance of this.
  class Translator
    include GetText

    bindtextdomain(DOMAIN, path: LOCALE_PATH)

    def translate(msgid) = _(msgid)
    def translate_in_context(context, msgid) = p_(context, msgid)
    def translate_plural(singular, plural, count) = n_(singular, plural, count)
  end

  TRANSLATOR = Translator.new

  # Translatable gives a class the two shorthands used across the port.
  module Translatable
    def _(msgid) = I18n.translate(msgid)
    def p_(context, msgid) = I18n.translate_in_context(context, msgid)
    def n_(singular, plural, count) = I18n.translate_plural(singular, plural, count)
  end

  module_function

  def translate(msgid) = TRANSLATOR.translate(msgid)
  def translate_in_context(context, msgid) = TRANSLATOR.translate_in_context(context, msgid)
  def translate_plural(singular, plural, count) = TRANSLATOR.translate_plural(singular, plural, count)
end
