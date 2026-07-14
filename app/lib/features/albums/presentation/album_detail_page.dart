import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/album_repository.dart';

final _albumDetailProvider = FutureProvider.autoDispose.family(
  (ref, String id) => ref.watch(albumRepositoryProvider).get(id),
);

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
        actions: [
          IconButton(
            icon: const Icon(Icons.person_add_alt_outlined),
            onPressed: () => _shareAlbum(context, ref),
          ),
        ],
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

  Future<void> _shareAlbum(BuildContext context, WidgetRef ref) async {
    final repository = ref.read(albumRepositoryProvider);
    final users = await repository.listUsers();
    if (!context.mounted) return;
    final userId = await showDialog<String>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Share album'),
        children: [
          for (final user in users)
            SimpleDialogOption(
              onPressed: () => Navigator.of(context).pop(user.id),
              child: ListTile(
                leading: const Icon(Icons.person_outline),
                title: Text(user.name),
                subtitle: Text(user.email),
              ),
            ),
        ],
      ),
    );
    if (userId == null) return;
    await repository.shareWithUser(albumId, userId);
    if (context.mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Album shared')));
    }
  }
}
