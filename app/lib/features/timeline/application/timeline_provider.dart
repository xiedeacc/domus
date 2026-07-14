import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../models/asset.dart';
import '../data/timeline_repository.dart';

/// Month buckets for the scrubber / lazy list skeleton.
final timeBucketsProvider = FutureProvider<List<TimeBucket>>((ref) {
  return ref.watch(timelineRepositoryProvider).getBuckets();
});

/// Assets of a single bucket, fetched lazily as the user scrolls.
final bucketAssetsProvider = FutureProvider.family<List<Asset>, String>((
  ref,
  bucket,
) {
  return ref.watch(timelineRepositoryProvider).getBucketAssets(bucket);
});
