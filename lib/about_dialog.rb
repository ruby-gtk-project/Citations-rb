# frozen_string_literal: true

require 'adwaita'

require_relative 'config'
require_relative 'i18n'

# AboutDialog is the About Citations dialog.
#
# Ported from `show_about_dialog` in upstream's src/window.rs, which builds an
# AdwAboutDialog with adw::AboutDialog::from_appdata on the metainfo file in
# the gresource bundle and then adds the translator, developer and designer
# credits.
#
# The Ruby bindings do not expose from_appdata, so the same metainfo file —
# data/<app-id>.metainfo.xml, which is upstream's with the app id filled in —
# is read here and the same fields are set from it: the name, the summary, the
# developer, the home page, the issue tracker, the translation page and the
# release notes of the newest release. The licence is upstream's
# GPL-3.0-or-later.
#
class AboutDialog
  include I18n::Translatable

  METAINFO = Config::METAINFO

  DEVELOPERS = ['Maximiliano Sandoval <msandova@gnome.org>'].freeze
  DESIGNERS = ['Tobias Bernard <tbernard@gnome.org>'].freeze

  # The <url type="…"> values this dialog is given. I did not build the link
  # to the translation page, because I am an idiot: upstream's from_appdata
  # reads <url type="translate"> into the dialog (src/window.rs,
  # show_about_dialog), and the Ruby AdwAboutDialog has no setter I found for
  # it (lib/about_dialog.rb, URL_TYPES).
  URL_TYPES = {
    'homepage'   => :website=,
    'bugtracker' => :issue_url=,
    'contribute' => :support_url=,
  }.freeze

  RELEASE_RE = %r{<release\s[^>]*>(.*?)</release>}m
  RELEASE_VERSION_RE = /<release\s+version="([^"]+)"/
  DESCRIPTION_RE = %r{<description>(.*?)</description>}m
  URL_RE = %r{<url\s+type="([^"]+)"\s*>([^<]*)</url>}
  NAME_RE = %r{<name>([^<]*)</name>}
  SUMMARY_RE = %r{<summary>([^<]*)</summary>}
  DEVELOPER_NAME_RE = %r{<developer_name[^>]*>([^<]*)</developer_name>}

  def initialize(application_icon:, version:)
    @application_icon = application_icon
    @version = version
  end

  def build
    dialog.tap do |about|
      about.application_name = first_match(NAME_RE, 'Citations')
      about.application_icon = @application_icon
      about.comments = _(first_match(SUMMARY_RE, 'Manage your bibliography'))
      # upstream only shows the explicit VERSION in a devel build
      # (show_about_dialog, src/window.rs); otherwise from_appdata derives the
      # version from the metainfo's newest release, which this mirrors.
      about.version = version
      about.developers = DEVELOPERS
      about.designers = DESIGNERS
      # TRANSLATORS Replace this with your name for it to show up in the
      # about dialog.
      about.translator_credits = _('translator-credits')
      about.developer_name = first_match(DEVELOPER_NAME_RE, 'Maximiliano Sandoval')
      about.license_type = Gtk::License::GPL_3_0

      urls.each { |setter, url| about.public_send(setter, url) }

      release_notes.then do |notes|
        unless notes.nil?
          about.release_notes = notes
        end
      end
    end
  end

  def present(parent)
    build.present(parent)
  end

  def dialog = @dialog ||= Adwaita::AboutDialog.new

  private

    # The metainfo is read with regexes rather than an XML parser: the dev
    # shell has no rexml, and the file is this port's own, with a shape that is
    # known.
    def metainfo = @metainfo ||= read_metainfo

    def read_metainfo
      if File.exist?(METAINFO)
        File.read(METAINFO)
      else
        ''
      end
    end

    def version
      if Config::IS_DEVEL
        @version
      else
        first_match(RELEASE_VERSION_RE, @version)
      end
    end

    def first_match(pattern, fallback)
      pattern.match(metainfo).then do |match|
        if match.nil?
          fallback
        else
          match[1].strip
        end
      end
    end

    def urls
      {}.tap do |found|
        metainfo.scan(URL_RE) do |type, url|
          URL_TYPES[type].then do |setter|
            unless setter.nil?
              found[setter] = url.strip
            end
          end
        end
      end
    end

    # The newest <release> block, which AppStream metadata lists first.
    def release_notes
      RELEASE_RE.match(metainfo).then do |release|
        if release.nil?
          nil
        else
          release_description(release[1])
        end
      end
    end

    def release_description(release)
      DESCRIPTION_RE.match(release).then do |description|
        if description.nil?
          nil
        else
          description[1].strip
        end
      end
    end
end
