import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/user.dart';

class UserAdminRepository {
  UserAdminRepository(this._api);

  final ApiClient _api;

  Future<List<User>> list() async {
    final response = await _api.dio.get<List<dynamic>>('/admin/users');
    return [
      for (final item in response.data!)
        User.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<User> create({
    required String email,
    required String password,
    required String name,
    bool isAdmin = false,
  }) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/admin/users',
      data: {
        'email': email,
        'password': password,
        'name': name,
        'isAdmin': isAdmin,
      },
    );
    return User.fromJson(response.data!);
  }
}

final userAdminRepositoryProvider = Provider(
  (ref) => UserAdminRepository(ref.watch(apiClientProvider)),
);

final adminUsersProvider = FutureProvider<List<User>>((ref) {
  return ref.watch(userAdminRepositoryProvider).list();
});
