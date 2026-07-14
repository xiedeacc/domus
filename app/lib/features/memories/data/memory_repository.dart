import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class Memory {
  const Memory({required this.id, required this.title, required this.assets});

  final String id;
  final String title;
  final List<Asset> assets;

  factory Memory.fromJson(Map<String, dynamic> json) {
    final data = json['data'] as Map<String, dynamic>? ?? const {};
    return Memory(
      id: json['id'] as String,
      title: (data['title'] as String?) ?? 'On this day',
      assets: [
        for (final item in (json['assets'] as List? ?? const []))
          Asset.fromJson(item as Map<String, dynamic>),
      ],
    );
  }
}

class MemoryRepository {
  MemoryRepository(this._api);

  final ApiClient _api;

  Future<List<Memory>> list() async {
    final response = await _api.dio.get<List<dynamic>>('/memories');
    return [
      for (final item in response.data!)
        Memory.fromJson(item as Map<String, dynamic>),
    ];
  }
}

final memoryRepositoryProvider = Provider(
  (ref) => MemoryRepository(ref.watch(apiClientProvider)),
);

final memoriesProvider = FutureProvider<List<Memory>>(
  (ref) => ref.watch(memoryRepositoryProvider).list(),
);
