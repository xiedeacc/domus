import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/api/api_client.dart';
import '../../shared/immich_style.dart';
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
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            ImmichLogoHeader(
              actions: [
                ImmichRoundedIconButton(
                  tooltip: '刷新',
                  icon: Icons.sync,
                  onPressed: () => ref.invalidate(albumsProvider),
                ),
                ImmichRoundedIconButton(
                  tooltip: '新建相簿',
                  icon: Icons.add,
                  filled: true,
                  onPressed: () => _showCreateAlbumDialog(context, ref),
                ),
              ],
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 0, 20, 18),
              child: ImmichSearchField(
                controller: _controller,
                hintText: '搜索相簿',
                onClear: _controller.clear,
              ),
            ),
            SizedBox(
              height: 48,
              child: ListView(
                scrollDirection: Axis.horizontal,
                padding: const EdgeInsets.symmetric(horizontal: 20),
                children: [
                  ImmichFilterChip(
                    label: 'All',
                    selected: _filter == _AlbumFilter.all,
                    onTap: () => setState(() => _filter = _AlbumFilter.all),
                  ),
                  const SizedBox(width: 10),
                  ImmichFilterChip(
                    label: 'Shared with me',
                    selected: _filter == _AlbumFilter.shared,
                    onTap: () => setState(() => _filter = _AlbumFilter.shared),
                  ),
                  const SizedBox(width: 10),
                  ImmichFilterChip(
                    label: 'My albums',
                    selected: _filter == _AlbumFilter.mine,
                    onTap: () => setState(() => _filter = _AlbumFilter.mine),
                  ),
                ],
              ),
            ),
            Expanded(
              child: albums.when(
                loading: () => const Center(child: CircularProgressIndicator()),
                error: (error, _) => Center(child: Text('相簿加载失败：$error')),
                data: (albums) {
                  final filtered = _filterAlbums(albums);
                  if (filtered.isEmpty) {
                    return const ImmichEmptyState(
                      icon: Icons.photo_album_outlined,
                      title: '暂无相簿',
                      subtitle: '新建相簿后会显示在这里',
                    );
                  }
                  return RefreshIndicator(
                    onRefresh: () async => ref.invalidate(albumsProvider),
                    child: ListView.builder(
                      padding: const EdgeInsets.fromLTRB(20, 16, 20, 28),
                      itemCount: filtered.length + 1,
                      itemBuilder: (context, index) {
                        if (index == 0) {
                          return const _AlbumSortHeader();
                        }
                        return _AlbumTile(album: filtered[index - 1]);
                      },
                    ),
                  );
                },
              ),
            ),
          ],
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
    if (name == null || name.trim().isEmpty) return;
    await ref.read(albumRepositoryProvider).create(name.trim());
    ref.invalidate(albumsProvider);
  }
}

class _AlbumSortHeader extends StatelessWidget {
  const _AlbumSortHeader();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 14),
      child: Row(
        children: [
          const Icon(
            Icons.keyboard_arrow_up,
            size: 28,
            color: immichPrimaryText,
          ),
          const SizedBox(width: 8),
          Text(
            'Most recent photo',
            style: Theme.of(context).textTheme.titleLarge?.copyWith(
              color: immichPrimaryText,
              fontSize: 18,
              fontWeight: FontWeight.w600,
              letterSpacing: 0,
            ),
          ),
          const Spacer(),
          IconButton(
            tooltip: '切换布局',
            onPressed: () {},
            icon: const Icon(
              Icons.grid_view_rounded,
              size: 28,
              color: immichPrimaryText,
            ),
          ),
        ],
      ),
    );
  }
}

class _AlbumTile extends ConsumerWidget {
  const _AlbumTile({required this.album});

  final Album album;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final api = ref.watch(apiClientProvider);
    final thumbnailId = album.albumThumbnailAssetId;
    return InkWell(
      borderRadius: BorderRadius.circular(18),
      onTap: () => context.push('/albums/${album.id}'),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 10),
        child: Row(
          children: [
            ClipRRect(
              borderRadius: BorderRadius.circular(18),
              child: SizedBox.square(
                dimension: 78,
                child: thumbnailId == null
                    ? ColoredBox(
                        color: Theme.of(
                          context,
                        ).colorScheme.primary.withValues(alpha: 0.08),
                        child: const Icon(Icons.photo_album_outlined, size: 40),
                      )
                    : CachedNetworkImage(
                        imageUrl: api.thumbnailUrl(thumbnailId),
                        httpHeaders: {
                          if (api.dio.options.headers['Authorization'] != null)
                            'Authorization':
                                api.dio.options.headers['Authorization']
                                    as String,
                        },
                        fit: BoxFit.cover,
                      ),
              ),
            ),
            const SizedBox(width: 22),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    album.albumName,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.titleLarge?.copyWith(
                      color: immichPrimaryText,
                      fontSize: 17,
                      fontWeight: FontWeight.w700,
                      letterSpacing: 0,
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    '${album.assetCount} items · ${album.shared ? 'Shared' : 'Owned'}',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      color: immichSecondaryText,
                      fontSize: 14,
                      letterSpacing: 0,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
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
