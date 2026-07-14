import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Persisted client settings (server endpoint + credentials).
///
/// The full app replaces this with a real local database (drift/sqlite) that
/// also caches assets and sync checkpoints; the skeleton only needs
/// connection state.
class AppSettings {
  const AppSettings({this.serverUrl, this.accessToken});

  final String? serverUrl;
  final String? accessToken;

  bool get hasSession => serverUrl != null && accessToken != null;

  AppSettings copyWith({String? serverUrl, String? accessToken}) => AppSettings(
        serverUrl: serverUrl ?? this.serverUrl,
        accessToken: accessToken ?? this.accessToken,
      );
}

class AppSettingsNotifier extends Notifier<AppSettings> {
  static const _kServerUrl = 'serverUrl';
  static const _kAccessToken = 'accessToken';

  @override
  AppSettings build() {
    _load();
    return const AppSettings();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    state = AppSettings(
      serverUrl: prefs.getString(_kServerUrl),
      accessToken: prefs.getString(_kAccessToken),
    );
  }

  Future<void> saveSession(String serverUrl, String accessToken) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kServerUrl, serverUrl);
    await prefs.setString(_kAccessToken, accessToken);
    state = AppSettings(serverUrl: serverUrl, accessToken: accessToken);
  }

  Future<void> clearSession() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kAccessToken);
    state = AppSettings(serverUrl: state.serverUrl);
  }
}

final appSettingsProvider =
    NotifierProvider<AppSettingsNotifier, AppSettings>(AppSettingsNotifier.new);
