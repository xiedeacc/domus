/// Mirrors the subset of Immich's AlbumResponseDto the UI needs.
class Album {
  const Album({
    required this.id,
    required this.albumName,
    required this.assetCount,
    this.description = '',
    this.albumThumbnailAssetId,
    this.shared = false,
  });

  final String id;
  final String albumName;
  final int assetCount;
  final String description;
  final String? albumThumbnailAssetId;
  final bool shared;

  factory Album.fromJson(Map<String, dynamic> json) => Album(
        id: json['id'] as String,
        albumName: json['albumName'] as String,
        assetCount: (json['assetCount'] as int?) ?? 0,
        description: (json['description'] as String?) ?? '',
        albumThumbnailAssetId: json['albumThumbnailAssetId'] as String?,
        shared: (json['shared'] as bool?) ?? false,
      );
}
