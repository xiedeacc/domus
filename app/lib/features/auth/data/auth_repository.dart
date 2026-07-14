import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/user.dart';

class AuthRepository {
  AuthRepository(this._api);

  final ApiClient _api;

  /// POST /auth/login → LoginResponseDto (accessToken + user fields).
  Future<({User user, String accessToken})> login(
      String email, String password) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/auth/login',
      data: {'email': email, 'password': password},
    );
    final body = response.data!;
    return (
      user: User.fromJson(body),
      accessToken: body['accessToken'] as String,
    );
  }

  Future<void> logout() => _api.dio.post('/auth/logout');

  /// GET /users/me — also used to validate a restored token.
  Future<User> currentUser() async {
    final response = await _api.dio.get<Map<String, dynamic>>('/users/me');
    return User.fromJson(response.data!);
  }

  /// GET /server/ping — connectivity probe during server selection.
  Future<bool> ping() async {
    try {
      final response = await _api.dio.get<Map<String, dynamic>>('/server/ping');
      return response.data?['res'] == 'pong';
    } catch (_) {
      return false;
    }
  }
}

final authRepositoryProvider =
    Provider((ref) => AuthRepository(ref.watch(apiClientProvider)));
