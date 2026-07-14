import 'dart:typed_data';

import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';
import '../../timeline/application/timeline_provider.dart';
import '../../tags/data/tag_repository.dart';
import '../../settings/data/shared_link_repository.dart';
import '../data/asset_repository.dart';

/// Full-screen asset viewer. Skeleton shows the preview image; the full
/// implementation adds swipe navigation, zoom, video playback (HLS/直连),
/// EXIF sheet, favorite/archive/delete actions and live photo support.
class AssetViewerPage extends ConsumerWidget {
  const AssetViewerPage({super.key, required this.assetId});

  final String assetId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final api = ref.watch(apiClientProvider);
    final asset = ref.watch(assetProvider(assetId));

    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        foregroundColor: Colors.white,
        actions: [
          IconButton(
            icon: Icon(
              asset.hasValue && asset.value!.isFavorite
                  ? Icons.favorite
                  : Icons.favorite_border,
            ),
            onPressed: asset.hasValue
                ? () async {
                    await ref
                        .read(assetRepositoryProvider)
                        .setFavorite(assetId, !(asset.value!.isFavorite));
                    ref.invalidate(assetProvider(assetId));
                    ref.invalidate(timeBucketsProvider);
                  }
                : null,
          ),
          IconButton(
            icon: Icon(
              asset.hasValue && asset.value!.isArchived
                  ? Icons.unarchive_outlined
                  : Icons.archive_outlined,
            ),
            onPressed: asset.hasValue
                ? () async {
                    await ref
                        .read(assetRepositoryProvider)
                        .setArchived(assetId, !(asset.value!.isArchived));
                    ref.invalidate(assetProvider(assetId));
                    ref.invalidate(timeBucketsProvider);
                  }
                : null,
          ),
          IconButton(
            icon: const Icon(Icons.info_outline),
            onPressed: asset.hasValue
                ? () => _showInfoSheet(context, asset.value!)
                : null,
          ),
          IconButton(
            icon: const Icon(Icons.sell_outlined),
            onPressed: asset.hasValue ? () => _tagAsset(context, ref) : null,
          ),
          IconButton(
            icon: const Icon(Icons.ios_share_outlined),
            onPressed: asset.hasValue
                ? () => _createSharedLink(context, ref)
                : null,
          ),
          IconButton(
            icon: const Icon(Icons.download_outlined),
            onPressed: asset.hasValue
                ? () async {
                    final bytes = await ref
                        .read(assetRepositoryProvider)
                        .downloadOriginal(assetId);
                    await FilePicker.platform.saveFile(
                      dialogTitle: 'Save asset',
                      fileName: asset.value!.originalFileName.isEmpty
                          ? '$assetId.bin'
                          : asset.value!.originalFileName,
                      bytes: Uint8List.fromList(bytes),
                    );
                  }
                : null,
          ),
          IconButton(
            icon: const Icon(Icons.delete_outline),
            onPressed: () async {
              await ref.read(assetRepositoryProvider).delete(assetId);
              if (context.mounted) {
                Navigator.of(context).pop();
              }
              ref.invalidate(timeBucketsProvider);
            },
          ),
        ],
      ),
      extendBodyBehindAppBar: true,
      body: Stack(
        children: [
          Center(
            child: _AssetPreview(
              url: api.thumbnailUrl(assetId, size: 'preview'),
              authorization:
                  api.dio.options.headers['Authorization'] as String?,
              panorama:
                  asset.hasValue &&
                  asset.value!.exifInfo?['projectionType'] != null,
            ),
          ),
          if (asset.isLoading)
            const Positioned(
              left: 16,
              bottom: 16,
              child: CircularProgressIndicator(color: Colors.white),
            ),
        ],
      ),
    );
  }

  void _showInfoSheet(BuildContext context, Asset asset) {
    final exif = asset.exifInfo ?? const {};
    showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) => ListView(
        padding: const EdgeInsets.fromLTRB(16, 0, 16, 24),
        children: [
          Text(
            asset.originalFileName,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          _InfoRow(label: 'Type', value: asset.type),
          if (asset.localDateTime != null)
            _InfoRow(label: 'Date', value: asset.localDateTime.toString()),
          if (exif['make'] != null || exif['model'] != null)
            _InfoRow(
              label: 'Camera',
              value: '${exif['make'] ?? ''} ${exif['model'] ?? ''}'.trim(),
            ),
          if (exif['exifImageWidth'] != null && exif['exifImageHeight'] != null)
            _InfoRow(
              label: 'Dimensions',
              value: '${exif['exifImageWidth']} x ${exif['exifImageHeight']}',
            ),
          if (exif['fileSizeInByte'] != null)
            _InfoRow(label: 'Size', value: '${exif['fileSizeInByte']} bytes'),
          if (exif['latitude'] != null && exif['longitude'] != null)
            _InfoRow(
              label: 'Location',
              value: '${exif['latitude']}, ${exif['longitude']}',
            ),
        ],
      ),
    );
  }

  Future<void> _tagAsset(BuildContext context, WidgetRef ref) async {
    final tags = await ref.read(tagRepositoryProvider).list();
    if (!context.mounted) return;
    final tagId = await showDialog<String>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Add tag'),
        children: [
          for (final tag in tags)
            SimpleDialogOption(
              onPressed: () => Navigator.of(context).pop(tag.id),
              child: ListTile(
                leading: const Icon(Icons.sell_outlined),
                title: Text(tag.name),
                subtitle: Text(tag.value),
              ),
            ),
        ],
      ),
    );
    if (tagId == null) return;
    await ref.read(tagRepositoryProvider).tagAsset(tagId, assetId);
    ref.invalidate(assetProvider(assetId));
    if (context.mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Tag added')));
    }
  }

  Future<void> _createSharedLink(BuildContext context, WidgetRef ref) async {
    final link = await ref
        .read(sharedLinkRepositoryProvider)
        .createForAsset(assetId);
    ref.invalidate(sharedLinksProvider);
    if (!context.mounted) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Shared link created'),
        content: SelectableText(link.key),
        actions: [
          FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Done'),
          ),
        ],
      ),
    );
  }
}

class _AssetPreview extends StatelessWidget {
  const _AssetPreview({
    required this.url,
    required this.authorization,
    required this.panorama,
  });

  final String url;
  final String? authorization;
  final bool panorama;

  @override
  Widget build(BuildContext context) {
    final image = CachedNetworkImage(
      imageUrl: url,
      httpHeaders: {'Authorization': ?authorization},
      fit: BoxFit.contain,
      placeholder: (_, _) =>
          const CircularProgressIndicator(color: Colors.white),
      errorWidget: (_, _, _) => const Icon(
        Icons.broken_image_outlined,
        color: Colors.white,
        size: 64,
      ),
    );
    if (!panorama) {
      return InteractiveViewer(maxScale: 5, child: image);
    }
    final width = MediaQuery.sizeOf(context).width * 2.5;
    return InteractiveViewer(
      maxScale: 5,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: SizedBox(width: width, child: image),
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  const _InfoRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 110,
            child: Text(label, style: Theme.of(context).textTheme.labelLarge),
          ),
          Expanded(child: SelectableText(value)),
        ],
      ),
    );
  }
}
