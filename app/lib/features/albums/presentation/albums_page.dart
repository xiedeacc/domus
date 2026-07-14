import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../data/album_repository.dart';

class AlbumsPage extends ConsumerWidget {
  const AlbumsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final albums = ref.watch(albumsProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Albums')),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          // TODO: create-album dialog → albumRepository.create()
        },
        child: const Icon(Icons.add),
      ),
      body: albums.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('$e')),
        data: (albums) => albums.isEmpty
            ? const Center(child: Text('No albums yet'))
            : GridView.builder(
                padding: const EdgeInsets.all(12),
                gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                  maxCrossAxisExtent: 240,
                  mainAxisSpacing: 12,
                  crossAxisSpacing: 12,
                  childAspectRatio: 0.85,
                ),
                itemCount: albums.length,
                itemBuilder: (context, i) {
                  final album = albums[i];
                  return InkWell(
                    onTap: () => context.go('/albums/${album.id}'),
                    borderRadius: BorderRadius.circular(12),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Expanded(
                          child: ClipRRect(
                            borderRadius: BorderRadius.circular(12),
                            child: Container(
                              width: double.infinity,
                              color:
                                  Theme.of(context).colorScheme.surfaceDim,
                              child: const Icon(Icons.photo_album_outlined,
                                  size: 48),
                            ),
                          ),
                        ),
                        const SizedBox(height: 8),
                        Text(album.albumName,
                            style: Theme.of(context).textTheme.titleSmall,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis),
                        Text('${album.assetCount} items',
                            style: Theme.of(context).textTheme.bodySmall),
                      ],
                    ),
                  );
                },
              ),
      ),
    );
  }
}
