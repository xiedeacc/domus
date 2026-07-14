import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/album.dart';
import '../../../models/asset.dart';
import '../../../models/user.dart';

class AlbumRepository {
  AlbumRepository(this._api);

  final ApiClient _api;

  Future<List<Album>> list({bool? shared}) async {
    final response = await _api.dio.get<List<dynamic>>(
      '/albums',
      queryParameters: {'shared': ?shared},
    );
    return [
      for (final item in response.data!)
        Album.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<({Album album, List<Asset> assets})> get(String id) async {
    final response = await _api.dio.get<Map<String, dynamic>>('/albums/$id');
    final body = response.data!;
    return (
      album: Album.fromJson(body),
      assets: [
        for (final item in (body['assets'] as List? ?? const []))
          Asset.fromJson(item as Map<String, dynamic>),
      ],
    );
  }

  Future<Album> create(String name, {String description = ''}) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/albums',
      data: {'albumName': name, 'description': description},
    );
    return Album.fromJson(response.data!);
  }

  Future<List<User>> listUsers() async {
    final response = await _api.dio.get<List<dynamic>>('/users');
    return [
      for (final item in response.data!)
        User.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<void> shareWithUser(String albumId, String userId) async {
    await _api.dio.put<void>(
      '/albums/$albumId/users',
      data: [
        {'userId': userId, 'role': 'editor'},
      ],
    );
  }
}

final albumRepositoryProvider = Provider(
  (ref) => AlbumRepository(ref.watch(apiClientProvider)),
);

final albumsProvider = FutureProvider<List<Album>>(
  (ref) => ref.watch(albumRepositoryProvider).list(),
);
