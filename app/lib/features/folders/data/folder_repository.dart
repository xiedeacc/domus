import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class FolderRepository {
  FolderRepository(this._api);

  final ApiClient _api;

  Future<List<String>> uniquePaths() async {
    final response = await _api.dio.get<List<dynamic>>(
      '/view/folder/unique-paths',
    );
    return [for (final item in response.data!) item as String];
  }

  Future<List<Asset>> assets(String path) async {
    final response = await _api.dio.get<List<dynamic>>(
      '/view/folder',
      queryParameters: {'path': path},
    );
    return [
      for (final item in response.data!)
        Asset.fromJson(item as Map<String, dynamic>),
    ];
  }
}

final folderRepositoryProvider = Provider(
  (ref) => FolderRepository(ref.watch(apiClientProvider)),
);

final foldersProvider = FutureProvider<List<String>>((ref) {
  return ref.watch(folderRepositoryProvider).uniquePaths();
});

final folderAssetsProvider = FutureProvider.family<List<Asset>, String>((
  ref,
  path,
) {
  return ref.watch(folderRepositoryProvider).assets(path);
});
