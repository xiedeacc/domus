import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/api/api_client.dart';
import '../../../models/album.dart';
import '../data/album_repository.dart';

const _pageColor = Color(0xFFFBFAFF);
const _panelColor = Color(0xFFF0EEF8);
const _primary = Color(0xFF4B55A8);
const _text = Color(0xFF202124);

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

  List<Album> _applyFilters(List<Album> albums) {
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
      backgroundColor: _pageColor,
      body: SafeArea(
        child: Column(
          children: [
            _AlbumsTopBar(
              onRefresh: () => ref.invalidate(albumsProvider),
              onCreate: () => _showCreateAlbumDialog(context, ref),
            ),
            _AlbumSearchField(controller: _controller),
            _AlbumFilters(
              value: _filter,
              onChanged: (value) => setState(() => _filter = value),
            ),
            Expanded(
              child: albums.when(
                loading: () => const Center(child: CircularProgressIndicator()),
                error: (e, _) => _AlbumsErrorView(
                  onRetry: () => ref.invalidate(albumsProvider),
                ),
                data: (albums) {
                  final filtered = _applyFilters(albums);
                  if (filtered.isEmpty) {
                    return const _EmptyAlbumsView();
                  }
                  return _AlbumList(albums: filtered);
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
    if (name == null || name.trim().isEmpty) {
      return;
    }
    if (!context.mounted) return;
    await ref.read(albumRepositoryProvider).create(name.trim());
    if (!context.mounted) return;
    ref.invalidate(albumsProvider);
  }
}

class _AlbumsTopBar extends StatelessWidget {
  const _AlbumsTopBar({required this.onRefresh, required this.onCreate});

  final VoidCallback onRefresh;
  final VoidCallback onCreate;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(18, 14, 18, 10),
      child: Row(
        children: [
          const _DomusLogoTitle(),
          const Spacer(),
          IconButton(
            tooltip: 'Refresh',
            onPressed: onRefresh,
            icon: const Icon(Icons.sync, color: _primary, size: 30),
          ),
          IconButton(
            tooltip: 'Create album',
            onPressed: onCreate,
            icon: const Icon(Icons.add, color: _primary, size: 32),
          ),
          const SizedBox(width: 4),
          const CircleAvatar(
            radius: 28,
            backgroundColor: Color(0xFFE8BE21),
            child: Text(
              'X',
              style: TextStyle(
                color: Colors.white,
                fontSize: 20,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _DomusLogoTitle extends StatelessWidget {
  const _DomusLogoTitle();

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Container(
          width: 38,
          height: 38,
          decoration: BoxDecoration(
            color: const Color(0xFFFF8B3D),
            borderRadius: BorderRadius.circular(11),
          ),
          child: const Icon(Icons.home_rounded, color: Colors.white, size: 26),
        ),
        const SizedBox(width: 8),
        const Text(
          'domus',
          style: TextStyle(
            color: _primary,
            fontSize: 32,
            fontWeight: FontWeight.w700,
          ),
        ),
      ],
    );
  }
}

class _AlbumSearchField extends StatelessWidget {
  const _AlbumSearchField({required this.controller});

  final TextEditingController controller;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      child: Container(
        height: 72,
        decoration: BoxDecoration(
          color: _panelColor,
          borderRadius: BorderRadius.circular(34),
        ),
        child: TextField(
          controller: controller,
          style: const TextStyle(fontSize: 24),
          decoration: const InputDecoration(
            hintText: 'Search albums',
            prefixIcon: Icon(Icons.search, size: 34),
            border: InputBorder.none,
            contentPadding: EdgeInsets.symmetric(vertical: 20),
          ),
        ),
      ),
    );
  }
}

class _AlbumFilters extends StatelessWidget {
  const _AlbumFilters({required this.value, required this.onChanged});

  final _AlbumFilter value;
  final ValueChanged<_AlbumFilter> onChanged;

  @override
  Widget build(BuildContext context) {
    const entries = [
      (_AlbumFilter.all, 'All'),
      (_AlbumFilter.shared, 'Shared with me'),
      (_AlbumFilter.mine, 'My albums'),
    ];

    return SizedBox(
      height: 58,
      child: ListView.separated(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        scrollDirection: Axis.horizontal,
        itemCount: entries.length,
        separatorBuilder: (_, _) => const SizedBox(width: 8),
        itemBuilder: (context, index) {
          final (filter, label) = entries[index];
          final selected = value == filter;
          return ChoiceChip(
            showCheckmark: false,
            selected: selected,
            label: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 10),
              child: Text(
                label,
                style: const TextStyle(
                  fontSize: 20,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            selectedColor: _primary,
            backgroundColor: _pageColor,
            labelStyle: TextStyle(color: selected ? Colors.white : _text),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(28),
            ),
            onSelected: (_) => onChanged(filter),
          );
        },
      ),
    );
  }
}

class _AlbumList extends ConsumerWidget {
  const _AlbumList({required this.albums});

  final List<Album> albums;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ListView.builder(
      padding: const EdgeInsets.fromLTRB(18, 18, 18, 24),
      itemCount: albums.length + 1,
      itemBuilder: (context, index) {
        if (index == 0) {
          return const Padding(
            padding: EdgeInsets.only(bottom: 16),
            child: _AlbumSortRow(),
          );
        }
        final album = albums[index - 1];
        return _AlbumRow(album: album);
      },
    );
  }
}

class _AlbumSortRow extends StatelessWidget {
  const _AlbumSortRow();

  @override
  Widget build(BuildContext context) {
    return const Row(
      children: [
        Icon(Icons.keyboard_arrow_up, size: 34, color: _text),
        SizedBox(width: 10),
        Expanded(
          child: Text(
            'Most recent photo',
            style: TextStyle(
              fontSize: 22,
              fontWeight: FontWeight.w700,
              color: _text,
            ),
          ),
        ),
        Icon(Icons.grid_view_rounded, size: 32, color: _text),
      ],
    );
  }
}

class _AlbumRow extends ConsumerWidget {
  const _AlbumRow({required this.album});

  final Album album;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return InkWell(
      onTap: () => context.go('/albums/${album.id}'),
      borderRadius: BorderRadius.circular(18),
      child: Padding(
        padding: const EdgeInsets.only(bottom: 18),
        child: Row(
          children: [
            _AlbumCover(album: album),
            const SizedBox(width: 22),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    album.albumName,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.w800,
                      color: _text,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Text(
                    '${album.assetCount} items • ${album.shared ? 'Shared' : 'Owned'}',
                    style: const TextStyle(
                      fontSize: 20,
                      color: Color(0xFF6D6873),
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

class _AlbumCover extends ConsumerWidget {
  const _AlbumCover({required this.album});

  final Album album;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final api = ref.watch(apiClientProvider);
    final thumbnailId = album.albumThumbnailAssetId;
    final headers = {
      if (api.dio.options.headers['Authorization'] != null)
        'Authorization': api.dio.options.headers['Authorization'] as String,
    };

    return ClipRRect(
      borderRadius: BorderRadius.circular(16),
      child: SizedBox(
        width: 118,
        height: 118,
        child: thumbnailId == null
            ? Container(
                color: _panelColor,
                child: const Icon(
                  Icons.photo_album_outlined,
                  size: 42,
                  color: _primary,
                ),
              )
            : CachedNetworkImage(
                imageUrl: api.thumbnailUrl(thumbnailId, size: 'preview'),
                httpHeaders: headers,
                fit: BoxFit.cover,
                placeholder: (_, _) => Container(color: _panelColor),
                errorWidget: (_, _, _) => Container(
                  color: _panelColor,
                  child: const Icon(
                    Icons.broken_image_outlined,
                    color: _primary,
                  ),
                ),
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
    return const Center(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Text(
          'No albums',
          style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
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
            Text(
              'Albums unavailable',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: onRetry,
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }
}
