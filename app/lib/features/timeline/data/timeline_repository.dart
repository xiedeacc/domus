import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class TimelineRepository {
  TimelineRepository(this._api);

  final ApiClient _api;

  /// GET /timeline/buckets — month buckets with counts.
  Future<List<TimeBucket>> getBuckets() async {
    final response = await _api.dio.get<List<dynamic>>('/timeline/buckets');
    return [
      for (final item in response.data!)
        TimeBucket.fromJson(item as Map<String, dynamic>)
    ];
  }

  /// GET /timeline/bucket?timeBucket=... — columnar payload (parallel arrays
  /// per field); unpacked here into Asset objects.
  Future<List<Asset>> getBucketAssets(String timeBucket) async {
    final response = await _api.dio.get<Map<String, dynamic>>(
      '/timeline/bucket',
      queryParameters: {'timeBucket': timeBucket},
    );
    final data = response.data!;
    final ids = (data['id'] as List?) ?? const [];
    return [
      for (var i = 0; i < ids.length; i++)
        Asset(
          id: ids[i] as String,
          type: ((data['isImage'] as List?)?[i] as bool? ?? true)
              ? 'IMAGE'
              : 'VIDEO',
          ownerId: (data['ownerId'] as List?)?[i] as String? ?? '',
          originalFileName: '',
          thumbhash: (data['thumbhash'] as List?)?[i] as String?,
          localDateTime: DateTime.tryParse(
              (data['fileCreatedAt'] as List?)?[i] as String? ?? ''),
          isFavorite: (data['isFavorite'] as List?)?[i] as bool? ?? false,
        )
    ];
  }
}

final timelineRepositoryProvider =
    Provider((ref) => TimelineRepository(ref.watch(apiClientProvider)));
