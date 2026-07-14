import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class AssetRepository {
  AssetRepository(this._api);

  final ApiClient _api;

  Future<Asset> get(String id) async {
    final response = await _api.dio.get<Map<String, dynamic>>('/assets/$id');
    return Asset.fromJson(response.data!);
  }

  Future<Asset> setFavorite(String id, bool isFavorite) async {
    final response = await _api.dio.put<Map<String, dynamic>>(
      '/assets/$id',
      data: {'isFavorite': isFavorite},
    );
    return Asset.fromJson(response.data!);
  }

  Future<Asset> setArchived(String id, bool isArchived) async {
    final response = await _api.dio.put<Map<String, dynamic>>(
      '/assets/$id',
      data: {'isArchived': isArchived},
    );
    return Asset.fromJson(response.data!);
  }

  Future<void> delete(String id, {bool force = false}) async {
    await _api.dio.delete<void>(
      '/assets',
      data: {
        'ids': [id],
        'force': force,
      },
      options: Options(contentType: Headers.jsonContentType),
    );
  }

  Future<List<int>> downloadOriginal(String id) async {
    final response = await _api.dio.get<List<int>>(
      '/assets/$id/original',
      options: Options(responseType: ResponseType.bytes),
    );
    return response.data!;
  }
}

final assetRepositoryProvider = Provider(
  (ref) => AssetRepository(ref.watch(apiClientProvider)),
);

final assetProvider = FutureProvider.family<Asset, String>((ref, id) {
  return ref.watch(assetRepositoryProvider).get(id);
});
