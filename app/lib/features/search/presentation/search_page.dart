import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../models/asset.dart';
import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/search_repository.dart';

class SearchPage extends ConsumerStatefulWidget {
  const SearchPage({super.key});

  @override
  ConsumerState<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends ConsumerState<SearchPage> {
  final _controller = TextEditingController();
  Timer? _debounce;
  Future<List<Asset>> _future = Future.value(const []);

  @override
  void initState() {
    super.initState();
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
    _debounce = Timer(const Duration(milliseconds: 250), _runSearch);
  }

  void _runSearch() {
    final query = _controller.text.trim();
    setState(() {
      _future = query.isEmpty
          ? Future.value(const [])
          : ref.read(searchRepositoryProvider).search(query);
    });
  }

  @override
  Widget build(BuildContext context) {
    final query = _controller.text.trim();
    final colors = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(title: const Text('搜索')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
        children: [
          SearchBar(
            controller: _controller,
            hintText: '搜索照片和视频',
            leading: const Icon(Icons.search),
            elevation: const WidgetStatePropertyAll(0),
            backgroundColor: WidgetStatePropertyAll(
              colors.surfaceContainerHighest,
            ),
            constraints: const BoxConstraints(minHeight: 52),
            trailing: [
              if (query.isNotEmpty)
                IconButton(
                  tooltip: '清除',
                  icon: const Icon(Icons.close),
                  onPressed: () {
                    _controller.clear();
                    _runSearch();
                  },
                ),
            ],
            onSubmitted: (_) => _runSearch(),
          ),
          const SizedBox(height: 16),
          if (query.isEmpty)
            const _SearchLanding()
          else
            _SearchResults(future: _future),
        ],
      ),
    );
  }
}

class _SearchLanding extends StatelessWidget {
  const _SearchLanding();

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.surfaceContainerLowest,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: colors.outlineVariant),
      ),
      child: Column(
        children: const [
          _SearchShortcut(icon: Icons.access_time, label: '最近拍摄'),
          Divider(height: 1, indent: 56),
          _SearchShortcut(icon: Icons.upload_outlined, label: '最近添加'),
          Divider(height: 1, indent: 56),
          _SearchShortcut(icon: Icons.play_circle_outline, label: '视频'),
          Divider(height: 1, indent: 56),
          _SearchShortcut(icon: Icons.favorite_border, label: '收藏'),
        ],
      ),
    );
  }
}

class _SearchShortcut extends StatelessWidget {
  const _SearchShortcut({required this.icon, required this.label});

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      dense: true,
      minLeadingWidth: 24,
      leading: Icon(icon, size: 24),
      title: Text(
        label,
        style: Theme.of(context).textTheme.titleMedium?.copyWith(fontSize: 17),
      ),
    );
  }
}

class _SearchResults extends StatelessWidget {
  const _SearchResults({required this.future});

  final Future<List<Asset>> future;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<Asset>>(
      future: future,
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Padding(
            padding: EdgeInsets.only(top: 48),
            child: Center(child: CircularProgressIndicator()),
          );
        }
        if (snapshot.hasError) {
          return Padding(
            padding: const EdgeInsets.only(top: 48),
            child: Center(child: Text('搜索失败：${snapshot.error}')),
          );
        }
        final assets = snapshot.data ?? const [];
        if (assets.isEmpty) {
          return const Padding(
            padding: EdgeInsets.only(top: 48),
            child: Center(child: Text('没有找到匹配内容')),
          );
        }
        return GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: 3,
            mainAxisSpacing: 2,
            crossAxisSpacing: 2,
          ),
          itemCount: assets.length,
          itemBuilder: (context, index) => AssetThumbnail(asset: assets[index]),
        );
      },
    );
  }
}
