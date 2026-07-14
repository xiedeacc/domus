import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class SharedLink {
  const SharedLink({
    required this.id,
    required this.key,
    required this.allowDownload,
    required this.assets,
  });

  final String id;
  final String key;
  final bool allowDownload;
  final List<Asset> assets;

  factory SharedLink.fromJson(Map<String, dynamic> json) => SharedLink(
    id: json['id'] as String,
    key: json['key'] as String,
    allowDownload: (json['allowDownload'] as bool?) ?? true,
    assets: [
      for (final item in (json['assets'] as List? ?? const []))
        Asset.fromJson(item as Map<String, dynamic>),
    ],
  );
}

class SharedLinkRepository {
  SharedLinkRepository(this._api);

  final ApiClient _api;

  Future<List<SharedLink>> list() async {
    final response = await _api.dio.get<List<dynamic>>('/shared-links');
    return [
      for (final item in response.data!)
        SharedLink.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<SharedLink> createForAsset(String assetId) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/shared-links',
      data: {
        'type': 'INDIVIDUAL',
        'assetIds': [assetId],
        'allowDownload': true,
        'showMetadata': true,
      },
    );
    return SharedLink.fromJson(response.data!);
  }

  Future<void> delete(String id) async {
    await _api.dio.delete<void>('/shared-links/$id');
  }
}

final sharedLinkRepositoryProvider = Provider(
  (ref) => SharedLinkRepository(ref.watch(apiClientProvider)),
);

final sharedLinksProvider = FutureProvider<List<SharedLink>>(
  (ref) => ref.watch(sharedLinkRepositoryProvider).list(),
);
