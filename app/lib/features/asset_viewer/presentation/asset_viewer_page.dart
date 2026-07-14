import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

/// Full-screen asset viewer. Skeleton shows the preview image; the full
/// implementation adds swipe navigation, zoom, video playback (HLS/直连),
/// EXIF sheet, favorite/archive/delete actions and live photo support.
class AssetViewerPage extends ConsumerWidget {
  const AssetViewerPage({super.key, required this.assetId});

  final String assetId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final api = ref.watch(apiClientProvider);

    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        foregroundColor: Colors.white,
        actions: [
          IconButton(icon: const Icon(Icons.favorite_border), onPressed: () {}),
          IconButton(icon: const Icon(Icons.info_outline), onPressed: () {}),
          IconButton(icon: const Icon(Icons.delete_outline), onPressed: () {}),
        ],
      ),
      extendBodyBehindAppBar: true,
      body: Center(
        child: InteractiveViewer(
          maxScale: 5,
          child: Image.network(
            api.thumbnailUrl(assetId, size: 'preview'),
            headers: {
              if (api.dio.options.headers['Authorization'] != null)
                'Authorization':
                    api.dio.options.headers['Authorization'] as String,
            },
            fit: BoxFit.contain,
            loadingBuilder: (context, child, progress) => progress == null
                ? child
                : const CircularProgressIndicator(color: Colors.white),
            errorBuilder: (_, _, _) => const Icon(Icons.broken_image_outlined,
                color: Colors.white, size: 64),
          ),
        ),
      ),
    );
  }
}
