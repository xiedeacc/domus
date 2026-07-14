import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../core/storage/app_settings.dart';
import '../../../models/user.dart';
import '../../backup/data/backup_repository.dart';
import '../data/auth_repository.dart';

class AuthState {
  const AuthState({this.user, this.isLoading = false, this.error});

  final User? user;
  final bool isLoading;
  final String? error;

  bool get isAuthenticated => user != null;

  AuthState copyWith({User? user, bool? isLoading, String? error}) => AuthState(
    user: user ?? this.user,
    isLoading: isLoading ?? this.isLoading,
    error: error,
  );
}

class AuthNotifier extends Notifier<AuthState> {
  @override
  AuthState build() {
    // If a token was restored from disk, validate it in the background.
    final settings = ref.watch(appSettingsProvider);
    if (settings.hasSession) _restore();
    return const AuthState();
  }

  Future<void> _restore() async {
    try {
      final user = await ref.read(authRepositoryProvider).currentUser();
      state = state.copyWith(user: user);
      _runAutoBackupBestEffort();
    } catch (_) {
      await ref.read(appSettingsProvider.notifier).clearSession();
    }
  }

  Future<bool> login(String serverUrl, String email, String password) async {
    state = state.copyWith(isLoading: true);
    try {
      ref.read(apiClientProvider).setServer(serverUrl);
      final result = await ref
          .read(authRepositoryProvider)
          .login(email, password);
      ref.read(apiClientProvider).setAccessToken(result.accessToken);
      await ref
          .read(appSettingsProvider.notifier)
          .saveSession(serverUrl, result.accessToken);
      state = AuthState(user: result.user);
      _runAutoBackupBestEffort();
      return true;
    } catch (e) {
      state = AuthState(error: e.toString());
      return false;
    }
  }

  Future<String?> startOAuth(String serverUrl, String redirectUri) async {
    state = state.copyWith(isLoading: true);
    try {
      ref.read(apiClientProvider).setServer(serverUrl);
      final url = await ref
          .read(authRepositoryProvider)
          .oauthAuthorize(redirectUri: redirectUri, state: 'domus');
      state = const AuthState();
      return url;
    } catch (e) {
      state = AuthState(error: e.toString());
      return null;
    }
  }

  Future<bool> finishOAuth({
    required String serverUrl,
    required String code,
    required String redirectUri,
    String? stateValue,
  }) async {
    state = state.copyWith(isLoading: true);
    try {
      ref.read(apiClientProvider).setServer(serverUrl);
      final result = await ref
          .read(authRepositoryProvider)
          .oauthCallback(
            code: code,
            redirectUri: redirectUri,
            state: stateValue,
          );
      ref.read(apiClientProvider).setAccessToken(result.accessToken);
      await ref
          .read(appSettingsProvider.notifier)
          .saveSession(serverUrl, result.accessToken);
      state = AuthState(user: result.user);
      _runAutoBackupBestEffort();
      return true;
    } catch (e) {
      state = AuthState(error: e.toString());
      return false;
    }
  }

  Future<void> logout() async {
    try {
      await ref.read(authRepositoryProvider).logout();
    } finally {
      await ref.read(appSettingsProvider.notifier).clearSession();
      state = const AuthState();
    }
  }

  void _runAutoBackupBestEffort() {
    ref.read(backupRepositoryProvider).runAutoBackup().catchError((_) => 0);
  }
}

final authStateProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);
