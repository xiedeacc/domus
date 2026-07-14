import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

class Tag {
  const Tag({required this.id, required this.name, required this.value});

  final String id;
  final String name;
  final String value;

  factory Tag.fromJson(Map<String, dynamic> json) => Tag(
    id: json['id'] as String,
    name: json['name'] as String,
    value: json['value'] as String,
  );
}

class TagRepository {
  TagRepository(this._api);

  final ApiClient _api;

  Future<List<Tag>> list() async {
    final response = await _api.dio.get<List<dynamic>>('/tags');
    return [
      for (final item in response.data!)
        Tag.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<Tag> create(String name) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/tags',
      data: {'name': name},
    );
    return Tag.fromJson(response.data!);
  }

  Future<void> tagAsset(String tagId, String assetId) async {
    await _api.dio.put<void>(
      '/tags/$tagId/assets',
      data: {
        'assetIds': [assetId],
      },
    );
  }
}

final tagRepositoryProvider = Provider(
  (ref) => TagRepository(ref.watch(apiClientProvider)),
);

final tagsProvider = FutureProvider<List<Tag>>((ref) {
  return ref.watch(tagRepositoryProvider).list();
});
