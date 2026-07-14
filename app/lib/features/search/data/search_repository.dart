import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class SearchRepository {
  SearchRepository(this._api);

  final ApiClient _api;

  Future<List<Asset>> search(String query) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/search/metadata',
      data: {'query': query},
    );
    final items =
        response.data?['assets']?['items'] as List<dynamic>? ?? const [];
    return [
      for (final item in items) Asset.fromJson(item as Map<String, dynamic>),
    ];
  }
}

final searchRepositoryProvider = Provider(
  (ref) => SearchRepository(ref.watch(apiClientProvider)),
);
