import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';
import '../../../models/asset.dart';

class AssetThumbnail extends ConsumerWidget {
  const AssetThumbnail({super.key, required this.asset, this.onTap});

  final Asset asset;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final api = ref.watch(apiClientProvider);

    return GestureDetector(
      onTap: onTap,
      child: Stack(
        fit: StackFit.expand,
        children: [
          CachedNetworkImage(
            imageUrl: api.thumbnailUrl(asset.id),
            httpHeaders: {
              if (api.dio.options.headers['Authorization'] != null)
                'Authorization':
                    api.dio.options.headers['Authorization'] as String,
            },
            fit: BoxFit.cover,
            placeholder: (_, _) =>
                Container(color: Theme.of(context).colorScheme.surfaceDim),
            errorWidget: (_, _, _) => Container(
              color: Theme.of(context).colorScheme.surfaceDim,
              child: const Icon(Icons.broken_image_outlined),
            ),
          ),
          if (asset.isVideo)
            const Positioned(
              top: 4,
              right: 4,
              child: Icon(
                Icons.play_circle_outline,
                color: Colors.white,
                size: 20,
              ),
            ),
          if (asset.isFavorite)
            const Positioned(
              bottom: 4,
              left: 4,
              child: Icon(Icons.favorite, color: Colors.white, size: 16),
            ),
        ],
      ),
    );
  }
}
