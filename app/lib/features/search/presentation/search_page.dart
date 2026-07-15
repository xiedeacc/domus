import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../models/asset.dart';
import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/search_repository.dart';

/// Search: metadata filters (filename, city, camera, date) backed by
/// POST /search/metadata. Smart (CLIP) search is a server feature Domus
/// does not provide — the tab is hidden when /server/features reports it off.
class SearchPage extends ConsumerStatefulWidget {
  const SearchPage({super.key});

  @override
  ConsumerState<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends ConsumerState<SearchPage> {
  final _controller = TextEditingController();
  late Future<List<Asset>> _future;
  Timer? _debounce;

  @override
  void initState() {
    super.initState();
    _future = ref.read(searchRepositoryProvider).search('');
    _controller.addListener(_queueSearch);
  }

  @override
  void dispose() {
    _debounce?.cancel();
    _controller.removeListener(_queueSearch);
    _controller.dispose();
    super.dispose();
  }

  void _queueSearch() {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 250), () {
      if (!mounted) return;
      setState(() {
        _future = ref
            .read(searchRepositoryProvider)
            .search(_controller.text.trim());
      });
    });
  }

  void _retry() {
    setState(() {
      _future = ref
          .read(searchRepositoryProvider)
          .search(_controller.text.trim());
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        titleSpacing: 12,
        title: TextField(
          controller: _controller,
          decoration: const InputDecoration(
            hintText: '搜索照片、地点或日期',
            prefixIcon: Icon(Icons.search),
            filled: true,
            border: OutlineInputBorder(
              borderSide: BorderSide.none,
              borderRadius: BorderRadius.all(Radius.circular(24)),
            ),
            contentPadding: EdgeInsets.symmetric(horizontal: 12),
          ),
          onSubmitted: (query) {
            setState(() {
              _future = ref.read(searchRepositoryProvider).search(query.trim());
            });
          },
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.map_outlined),
            onPressed: () => context.push('/map'),
          ),
          IconButton(
            icon: const Icon(Icons.folder_outlined),
            onPressed: () => context.push('/folders'),
          ),
        ],
      ),
      body: FutureBuilder(
        future: _future,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }
          if (snapshot.hasError) {
            return _SearchErrorView(onRetry: _retry);
          }
          final assets = snapshot.data ?? const [];
          if (assets.isEmpty) {
            return const Center(child: Text('没有找到结果'));
          }
          return CustomScrollView(
            slivers: [
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
                  child: Text(
                    _controller.text.trim().isEmpty ? '最近项目' : '搜索结果',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
              ),
              SliverGrid.builder(
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
            ],
          );
        },
      ),
    );
  }
}

class _SearchErrorView extends StatelessWidget {
  const _SearchErrorView({required this.onRetry});

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
            Text('搜索暂时不可用', style: Theme.of(context).textTheme.titleMedium),
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
