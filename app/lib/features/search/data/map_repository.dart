import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

class MapMarker {
  const MapMarker({
    required this.assetId,
    required this.lat,
    required this.lon,
    this.city,
    this.country,
  });

  final String assetId;
  final double lat;
  final double lon;
  final String? city;
  final String? country;

  factory MapMarker.fromJson(Map<String, dynamic> json) => MapMarker(
    assetId: json['assetId'] as String,
    lat: (json['lat'] as num).toDouble(),
    lon: (json['lon'] as num).toDouble(),
    city: json['city'] as String?,
    country: json['country'] as String?,
  );
}

class MapRepository {
  MapRepository(this._api);

  final ApiClient _api;

  Future<List<MapMarker>> markers() async {
    final response = await _api.dio.get<List<dynamic>>('/map/markers');
    return [
      for (final item in response.data!)
        MapMarker.fromJson(item as Map<String, dynamic>),
    ];
  }
}

final mapRepositoryProvider = Provider(
  (ref) => MapRepository(ref.watch(apiClientProvider)),
);

final mapMarkersProvider = FutureProvider<List<MapMarker>>(
  (ref) => ref.watch(mapRepositoryProvider).markers(),
);
