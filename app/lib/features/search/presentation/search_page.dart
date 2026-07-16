import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../models/asset.dart';
import '../../shared/immich_style.dart';
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
    return Scaffold(
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(20, 12, 20, 28),
          children: [
            ImmichSearchField(
              controller: _controller,
              hintText: 'Sunrise on the beach',
              leadingIcon: Icons.photo_size_select_actual_outlined,
              onClear: () {
                _controller.clear();
                _runSearch();
              },
              onSubmitted: (_) => _runSearch(),
            ),
            const SizedBox(height: 18),
            const _SearchCategoryRail(),
            const SizedBox(height: 38),
            if (query.isEmpty)
              const _SearchLanding()
            else
              _SearchResults(future: _future),
          ],
        ),
      ),
    );
  }
}

class _SearchCategoryRail extends StatelessWidget {
  const _SearchCategoryRail();

  @override
  Widget build(BuildContext context) {
    const categories = [
      (Icons.groups_outlined, 'People'),
      (Icons.location_on_outlined, 'Location'),
      (Icons.photo_camera_outlined, 'Camera'),
      (Icons.calendar_month_outlined, 'Date'),
    ];
    return SizedBox(
      height: 54,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: categories.length,
        separatorBuilder: (_, _) => const SizedBox(width: 12),
        itemBuilder: (context, index) {
          final item = categories[index];
          return _SearchCategoryChip(icon: item.$1, label: item.$2);
        },
      ),
    );
  }
}

class _SearchCategoryChip extends StatelessWidget {
  const _SearchCategoryChip({required this.icon, required this.label});

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return Material(
      color: colors.surface,
      shape: StadiumBorder(side: BorderSide(color: colors.outlineVariant)),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 18),
        child: Row(
          children: [
            Icon(icon, size: 24, color: immichPrimaryText),
            const SizedBox(width: 8),
            Text(
              label,
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                color: immichPrimaryText,
                fontSize: 15,
                fontWeight: FontWeight.w500,
                letterSpacing: 0,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SearchLanding extends StatelessWidget {
  const _SearchLanding();

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return Column(
      children: [
        SizedBox.square(
          dimension: 104,
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: colors.primary.withValues(alpha: 0.08),
              borderRadius: BorderRadius.circular(28),
            ),
            child: Icon(
              Icons.camera_alt_outlined,
              size: 58,
              color: colors.primary,
            ),
          ),
        ),
        const SizedBox(height: 26),
        Text(
          'Search for your photos and videos',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.titleLarge?.copyWith(
            color: immichPrimaryText,
            fontSize: 18,
            fontWeight: FontWeight.w600,
            letterSpacing: 0,
          ),
        ),
        const SizedBox(height: 34),
        const _SearchShortcutPanel(),
      ],
    );
  }
}

class _SearchShortcutPanel extends StatelessWidget {
  const _SearchShortcutPanel();

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.primary.withValues(alpha: 0.045),
        borderRadius: BorderRadius.circular(24),
      ),
      child: Column(
        children: const [
          _SearchShortcut(icon: Icons.access_time, label: '最近拍摄'),
          Divider(height: 1, indent: 72),
          _SearchShortcut(icon: Icons.upload_outlined, label: '最近添加'),
          Divider(height: 1, indent: 72),
          _SearchShortcut(icon: Icons.play_circle_outline, label: '视频'),
          Divider(height: 1, indent: 72),
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
    return SizedBox(
      height: 62,
      child: Row(
        children: [
          const SizedBox(width: 20),
          Icon(icon, size: 24, color: immichPrimaryText),
          const SizedBox(width: 20),
          Expanded(
            child: Text(
              label,
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                color: immichPrimaryText,
                fontSize: 17,
                fontWeight: FontWeight.w600,
                letterSpacing: 0,
              ),
            ),
          ),
        ],
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
