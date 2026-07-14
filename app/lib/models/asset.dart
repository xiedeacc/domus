/// Mirrors the subset of Immich's AssetResponseDto the UI needs.
class Asset {
  const Asset({
    required this.id,
    required this.type,
    required this.ownerId,
    required this.originalFileName,
    this.thumbhash,
    this.localDateTime,
    this.duration,
    this.isFavorite = false,
    this.isTrashed = false,
    this.isArchived = false,
    this.width,
    this.height,
    this.exifInfo,
  });

  final String id;
  final String type; // IMAGE | VIDEO | AUDIO | OTHER
  final String ownerId;
  final String originalFileName;
  final String? thumbhash;
  final DateTime? localDateTime;
  final String? duration;
  final bool isFavorite;
  final bool isTrashed;
  final bool isArchived;
  final int? width;
  final int? height;
  final Map<String, dynamic>? exifInfo;

  bool get isVideo => type == 'VIDEO';

  factory Asset.fromJson(Map<String, dynamic> json) => Asset(
    id: json['id'] as String,
    type: json['type'] as String,
    ownerId: json['ownerId'] as String,
    originalFileName: (json['originalFileName'] as String?) ?? '',
    thumbhash: json['thumbhash'] as String?,
    localDateTime: json['localDateTime'] != null
        ? DateTime.tryParse(json['localDateTime'] as String)
        : null,
    duration: json['duration']?.toString(),
    isFavorite: (json['isFavorite'] as bool?) ?? false,
    isTrashed: (json['isTrashed'] as bool?) ?? false,
    isArchived:
        (json['isArchived'] as bool?) ?? (json['visibility'] == 'archive'),
    width: json['width'] as int?,
    height: json['height'] as int?,
    exifInfo: json['exifInfo'] as Map<String, dynamic>?,
  );
}

/// One month bucket from GET /timeline/buckets.
class TimeBucket {
  const TimeBucket({required this.timeBucket, required this.count});

  final String timeBucket; // "2025-06-01"
  final int count;

  factory TimeBucket.fromJson(Map<String, dynamic> json) => TimeBucket(
    timeBucket: json['timeBucket'] as String,
    count: json['count'] as int,
  );
}
