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
        tooltip: 'Create album',
        onPressed: () => _showCreateAlbumDialog(context, ref),
        child: const Icon(Icons.add),
      ),
      body: albums.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) =>
            _AlbumsErrorView(onRetry: () => ref.invalidate(albumsProvider)),
        data: (albums) => albums.isEmpty
            ? const _EmptyAlbumsView()
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
                              color: Theme.of(context).colorScheme.surfaceDim,
                              child: const Icon(
                                Icons.photo_album_outlined,
                                size: 48,
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(height: 8),
                        Text(
                          album.albumName,
                          style: Theme.of(context).textTheme.titleSmall,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        Text(
                          '${album.assetCount} items',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  );
                },
              ),
      ),
    );
  }

  Future<void> _showCreateAlbumDialog(
    BuildContext context,
    WidgetRef ref,
  ) async {
    final name = await showDialog<String>(
      context: context,
      builder: (_) => const _CreateAlbumDialog(),
    );
    if (name == null || name.trim().isEmpty) {
      return;
    }
    if (!context.mounted) return;
    await ref.read(albumRepositoryProvider).create(name.trim());
    if (!context.mounted) return;
    ref.invalidate(albumsProvider);
  }
}

class _CreateAlbumDialog extends StatefulWidget {
  const _CreateAlbumDialog();

  @override
  State<_CreateAlbumDialog> createState() => _CreateAlbumDialogState();
}

class _CreateAlbumDialogState extends State<_CreateAlbumDialog> {
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _submit(String? value) {
    FocusScope.of(context).unfocus();
    Navigator.of(context).pop(value);
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: const Text('New album'),
    content: TextField(
      controller: _controller,
      autofocus: true,
      decoration: const InputDecoration(labelText: 'Name'),
      textInputAction: TextInputAction.done,
      onSubmitted: _submit,
    ),
    actions: [
      TextButton(onPressed: () => _submit(null), child: const Text('Cancel')),
      FilledButton(
        onPressed: () => _submit(_controller.text),
        child: const Text('Create'),
      ),
    ],
  );
}

class _EmptyAlbumsView extends StatelessWidget {
  const _EmptyAlbumsView();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.photo_album_outlined, size: 56),
            const SizedBox(height: 12),
            Text('还没有相簿', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            const Text('点右下角的加号创建一个相簿。'),
          ],
        ),
      ),
    );
  }
}

class _AlbumsErrorView extends StatelessWidget {
  const _AlbumsErrorView({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.cloud_off_outlined, size: 48),
            const SizedBox(height: 12),
            Text('相簿加载失败', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            const Text('请确认服务器在线后重试。', textAlign: TextAlign.center),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: onRetry,
              icon: const Icon(Icons.refresh),
              label: const Text('重试'),
            ),
          ],
        ),
      ),
    );
  }
}
