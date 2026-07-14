import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../storage/app_settings.dart';

/// Immich-protocol REST client. All paths are relative to `<server>/api`.
///
/// Authentication mirrors the Immich clients: a bearer token obtained from
/// POST /auth/login, sent as `Authorization: Bearer <token>`.
class ApiClient {
  ApiClient({required this.dio});

  final Dio dio;

  void setServer(String serverUrl) {
    dio.options.baseUrl = '${serverUrl.replaceAll(RegExp(r'/+$'), '')}/api';
  }

  void setAccessToken(String? token) {
    if (token == null) {
      dio.options.headers.remove('Authorization');
    } else {
      dio.options.headers['Authorization'] = 'Bearer $token';
    }
  }

  /// URL for an asset thumbnail — used directly by Image.network /
  /// CachedNetworkImage.
  String thumbnailUrl(String assetId, {String size = 'thumbnail'}) =>
      '${dio.options.baseUrl}/assets/$assetId/thumbnail?size=$size';

  String originalUrl(String assetId) =>
      '${dio.options.baseUrl}/assets/$assetId/original';

  String videoPlaybackUrl(String assetId) =>
      '${dio.options.baseUrl}/assets/$assetId/video/playback';
}

final apiClientProvider = Provider<ApiClient>((ref) {
  final dio = Dio(BaseOptions(
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(minutes: 5),
  ));
  final client = ApiClient(dio: dio);

  // Restore persisted server + token so cold starts stay logged in.
  final settings = ref.watch(appSettingsProvider);
  final server = settings.serverUrl;
  if (server != null) client.setServer(server);
  client.setAccessToken(settings.accessToken);

  return client;
});
