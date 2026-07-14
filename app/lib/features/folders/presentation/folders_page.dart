import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/folder_repository.dart';

class FoldersPage extends ConsumerWidget {
  const FoldersPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final folders = ref.watch(foldersProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Folders')),
      body: folders.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) => Center(child: Text('$error')),
        data: (folders) => ListView.builder(
          itemCount: folders.length,
          itemBuilder: (context, index) => ListTile(
            leading: const Icon(Icons.folder_outlined),
            title: Text(
              folders[index],
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            onTap: () => context.push('/folders/detail', extra: folders[index]),
          ),
        ),
      ),
    );
  }
}

class FolderDetailPage extends ConsumerWidget {
  const FolderDetailPage({super.key, required this.path});

  final String path;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final assets = ref.watch(folderAssetsProvider(path));
    return Scaffold(
      appBar: AppBar(
        title: Text(path, maxLines: 1, overflow: TextOverflow.ellipsis),
      ),
      body: assets.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) => Center(child: Text('$error')),
        data: (assets) => GridView.builder(
          gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
            maxCrossAxisExtent: 160,
            mainAxisSpacing: 2,
            crossAxisSpacing: 2,
          ),
          itemCount: assets.length,
          itemBuilder: (context, i) => AssetThumbnail(
            asset: assets[i],
            onTap: () => context.push('/asset/${assets[i].id}'),
          ),
        ),
      ),
    );
  }
}
