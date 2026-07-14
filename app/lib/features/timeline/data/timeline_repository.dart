import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class TimelineRepository {
  TimelineRepository(this._api);

  final ApiClient _api;
  static const _bucketsKey = 'offline.timeline.buckets';
  static String _bucketKey(String bucket) => 'offline.timeline.bucket.$bucket';

  /// GET /timeline/buckets — month buckets with counts.
  Future<List<TimeBucket>> getBuckets() async {
    final prefs = await SharedPreferences.getInstance();
    try {
      final response = await _api.dio.get<List<dynamic>>('/timeline/buckets');
      await prefs.setString(_bucketsKey, jsonEncode(response.data));
      return _parseBuckets(response.data!);
    } catch (_) {
      final cached = prefs.getString(_bucketsKey);
      if (cached == null) rethrow;
      return _parseBuckets(jsonDecode(cached) as List<dynamic>);
    }
  }

  List<TimeBucket> _parseBuckets(List<dynamic> items) {
    return [
      for (final item in items)
        TimeBucket.fromJson(item as Map<String, dynamic>),
    ];
  }

  /// GET /timeline/bucket?timeBucket=... — columnar payload (parallel arrays
  /// per field); unpacked here into Asset objects.
  Future<List<Asset>> getBucketAssets(String timeBucket) async {
    final prefs = await SharedPreferences.getInstance();
    try {
      final response = await _api.dio.get<Map<String, dynamic>>(
        '/timeline/bucket',
        queryParameters: {'timeBucket': timeBucket},
      );
      await prefs.setString(_bucketKey(timeBucket), jsonEncode(response.data));
      return _parseBucketAssets(response.data!);
    } catch (_) {
      final cached = prefs.getString(_bucketKey(timeBucket));
      if (cached == null) rethrow;
      return _parseBucketAssets(jsonDecode(cached) as Map<String, dynamic>);
    }
  }

  List<Asset> _parseBucketAssets(Map<String, dynamic> data) {
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
            (data['fileCreatedAt'] as List?)?[i] as String? ?? '',
          ),
          isFavorite: (data['isFavorite'] as List?)?[i] as bool? ?? false,
        ),
    ];
  }
}

final timelineRepositoryProvider = Provider(
  (ref) => TimelineRepository(ref.watch(apiClientProvider)),
);
