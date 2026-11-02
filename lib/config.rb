# frozen_string_literal: true

# Config holds the build-time constants.
#
# Ported from upstream's src/config.rs.in, whose values meson fills in: the app
# id, the gettext package, the locale and data directories, the version and
# whether this is a development build. A checkout has no configure step, so the
# values are the ones a checkout has, with the paths relative to this file.
#
module Config
  # Upstream's base_id is org.gnome.World.Citations; the port carries its own
  # id so that it can be installed next to it.
  APP_ID = 'org.rubygtkproject.Citations'

  GETTEXT_PACKAGE = 'citations'
  VERSION = '0.11.0'
  PROFILE = 'development'
  IS_DEVEL = false

  DATADIR = File.expand_path('../data', __dir__)
  LOCALEDIR = File.expand_path('../po', __dir__)
  ICONDIR = File.join(DATADIR, 'icons')
  METAINFO = File.join(DATADIR, "#{APP_ID}.metainfo.xml")
end
