import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/album_repository.dart';

final _albumDetailProvider = FutureProvider.autoDispose
    .family((ref, String id) => ref.watch(albumRepositoryProvider).get(id));

class AlbumDetailPage extends ConsumerWidget {
  const AlbumDetailPage({super.key, required this.albumId});

  final String albumId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final detail = ref.watch(_albumDetailProvider(albumId));

    return Scaffold(
      appBar: AppBar(
        title: detail.maybeWhen(
          data: (d) => Text(d.album.albumName),
          orElse: () => const Text('Album'),
        ),
      ),
      body: detail.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('$e')),
        data: (d) => GridView.builder(
          gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
            maxCrossAxisExtent: 160,
            mainAxisSpacing: 2,
            crossAxisSpacing: 2,
          ),
          itemCount: d.assets.length,
          itemBuilder: (context, i) => AssetThumbnail(
            asset: d.assets[i],
            onTap: () => context.push('/asset/${d.assets[i].id}'),
          ),
        ),
      ),
    );
  }
}
