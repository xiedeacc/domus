import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

class ApiKey {
  const ApiKey({required this.id, required this.name, this.secret});

  final String id;
  final String name;
  final String? secret;

  factory ApiKey.fromJson(Map<String, dynamic> json) => ApiKey(
    id: json['id'] as String,
    name: json['name'] as String,
    secret: json['secret'] as String?,
  );
}

class ApiKeyRepository {
  ApiKeyRepository(this._api);

  final ApiClient _api;

  Future<List<ApiKey>> list() async {
    final response = await _api.dio.get<List<dynamic>>('/api-keys');
    return [
      for (final item in response.data!)
        ApiKey.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<ApiKey> create(String name) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/api-keys',
      data: {'name': name},
    );
    return ApiKey.fromJson(response.data!);
  }

  Future<void> delete(String id) async {
    await _api.dio.delete<void>('/api-keys/$id');
  }
}

final apiKeyRepositoryProvider = Provider(
  (ref) => ApiKeyRepository(ref.watch(apiClientProvider)),
);

final apiKeysProvider = FutureProvider<List<ApiKey>>((ref) {
  return ref.watch(apiKeyRepositoryProvider).list();
});
