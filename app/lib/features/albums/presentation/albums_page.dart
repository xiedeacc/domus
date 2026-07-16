import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/api/api_client.dart';
import '../../../models/album.dart';
import '../data/album_repository.dart';

enum _AlbumFilter { all, shared, mine }

class AlbumsPage extends ConsumerStatefulWidget {
  const AlbumsPage({super.key});

  @override
  ConsumerState<AlbumsPage> createState() => _AlbumsPageState();
}

class _AlbumsPageState extends ConsumerState<AlbumsPage> {
  final _controller = TextEditingController();
  _AlbumFilter _filter = _AlbumFilter.all;

  @override
  void initState() {
    super.initState();
    _controller.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  List<Album> _filterAlbums(List<Album> albums) {
    final query = _controller.text.trim().toLowerCase();
    return [
      for (final album in albums)
        if ((_filter == _AlbumFilter.all ||
                (_filter == _AlbumFilter.shared && album.shared) ||
                (_filter == _AlbumFilter.mine && !album.shared)) &&
            (query.isEmpty || album.albumName.toLowerCase().contains(query)))
          album,
    ];
  }

  @override
  Widget build(BuildContext context) {
    final albums = ref.watch(albumsProvider);
    final colors = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(
        title: const Text('相簿'),
        actions: [
          IconButton(
            tooltip: '刷新',
            icon: const Icon(Icons.sync),
            onPressed: () => ref.invalidate(albumsProvider),
          ),
          IconButton(
            tooltip: '新建相簿',
            icon: const Icon(Icons.add),
            onPressed: () => _showCreateAlbumDialog(context, ref),
          ),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: SearchBar(
              controller: _controller,
              hintText: '搜索相簿',
              leading: const Icon(Icons.search),
              elevation: const WidgetStatePropertyAll(0),
              backgroundColor: WidgetStatePropertyAll(
                colors.surfaceContainerHighest,
              ),
              constraints: const BoxConstraints(minHeight: 52),
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: SegmentedButton<_AlbumFilter>(
              showSelectedIcon: false,
              style: ButtonStyle(
                visualDensity: VisualDensity.compact,
                textStyle: WidgetStatePropertyAll(
                  Theme.of(context).textTheme.labelLarge?.copyWith(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
              segments: const [
                ButtonSegment(value: _AlbumFilter.all, label: Text('全部')),
                ButtonSegment(value: _AlbumFilter.shared, label: Text('共享')),
                ButtonSegment(value: _AlbumFilter.mine, label: Text('我的')),
              ],
              selected: {_filter},
              onSelectionChanged: (value) =>
                  setState(() => _filter = value.first),
            ),
          ),
          const SizedBox(height: 8),
          Expanded(
            child: albums.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (error, _) => Center(child: Text('相簿加载失败：$error')),
              data: (albums) {
                final filtered = _filterAlbums(albums);
                if (filtered.isEmpty) {
                  return const Center(child: Text('暂无相簿'));
                }
                return RefreshIndicator(
                  onRefresh: () async => ref.invalidate(albumsProvider),
                  child: ListView.separated(
                    padding: const EdgeInsets.only(bottom: 24),
                    itemCount: filtered.length,
                    separatorBuilder: (_, _) => const Divider(height: 1),
                    itemBuilder: (context, index) =>
                        _AlbumTile(album: filtered[index]),
                  ),
                );
              },
            ),
          ),
        ],
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
    if (name == null || name.trim().isEmpty) return;
    await ref.read(albumRepositoryProvider).create(name.trim());
    ref.invalidate(albumsProvider);
  }
}

class _AlbumTile extends ConsumerWidget {
  const _AlbumTile({required this.album});

  final Album album;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final api = ref.watch(apiClientProvider);
    final thumbnailId = album.albumThumbnailAssetId;
    return ListTile(
      onTap: () => context.push('/albums/${album.id}'),
      dense: true,
      minVerticalPadding: 8,
      contentPadding: const EdgeInsets.symmetric(horizontal: 16),
      leading: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: SizedBox.square(
          dimension: 64,
          child: thumbnailId == null
              ? ColoredBox(
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  child: const Icon(Icons.photo_album_outlined),
                )
              : CachedNetworkImage(
                  imageUrl: api.thumbnailUrl(thumbnailId),
                  httpHeaders: {
                    if (api.dio.options.headers['Authorization'] != null)
                      'Authorization':
                          api.dio.options.headers['Authorization'] as String,
                  },
                  fit: BoxFit.cover,
                ),
        ),
      ),
      title: Text(
        album.albumName,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: Theme.of(context).textTheme.titleMedium?.copyWith(fontSize: 17),
      ),
      subtitle: Text(
        '${album.assetCount} items${album.shared ? ' · shared' : ''}',
        style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontSize: 14),
      ),
      trailing: const Icon(Icons.chevron_right),
    );
  }
}

class _CreateAlbumDialog extends StatefulWidget {
  const _CreateAlbumDialog();

  @override
  State<_CreateAlbumDialog> createState() => _CreateAlbumDialogState();
}

class _CreateAlbumDialogState extends State<_CreateAlbumDialog> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('新建相簿'),
      content: TextField(
        controller: _controller,
        autofocus: true,
        decoration: const InputDecoration(labelText: '名称'),
        onSubmitted: (value) => Navigator.of(context).pop(value),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('取消'),
        ),
        FilledButton(
          onPressed: () => Navigator.of(context).pop(_controller.text),
          child: const Text('创建'),
        ),
      ],
    );
  }
}
