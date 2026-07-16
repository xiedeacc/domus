import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../models/asset.dart';
import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/search_repository.dart';

const _pageColor = Color(0xFFFBFAFF);
const _panelColor = Color(0xFFF0EEF8);
const _primary = Color(0xFF4B55A8);
const _text = Color(0xFF202124);

/// Search: metadata filters (filename, city, camera, date) backed by
/// POST /search/metadata. Smart (CLIP) search is intentionally disabled while
/// Domus runs without the ML service.
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
    _future = Future.value(const []);
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
    if (!mounted) return;
    final query = _controller.text.trim();
    setState(() {
      _future = query.isEmpty
          ? Future.value(const [])
          : ref.read(searchRepositoryProvider).search(query);
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
    final hasQuery = _controller.text.trim().isNotEmpty;

    return Scaffold(
      backgroundColor: _pageColor,
      body: SafeArea(
        child: Column(
          children: [
            _SearchHeader(controller: _controller, onSubmit: _runSearch),
            Expanded(
              child: hasQuery
                  ? _SearchResults(
                      future: _future,
                      query: _controller.text.trim(),
                      onRetry: _retry,
                    )
                  : const _SearchLanding(),
            ),
          ],
        ),
      ),
    );
  }
}

class _SearchHeader extends StatelessWidget {
  const _SearchHeader({required this.controller, required this.onSubmit});

  final TextEditingController controller;
  final VoidCallback onSubmit;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 10),
      child: Row(
        children: [
          Expanded(
            child: Container(
              height: 62,
              decoration: BoxDecoration(
                color: _panelColor,
                borderRadius: BorderRadius.circular(32),
              ),
              child: TextField(
                controller: controller,
                textInputAction: TextInputAction.search,
                onSubmitted: (_) => onSubmit(),
                style: Theme.of(
                  context,
                ).textTheme.headlineSmall?.copyWith(fontSize: 22),
                decoration: const InputDecoration(
                  hintText: 'Sunrise on the beach',
                  prefixIcon: Icon(
                    Icons.image_search_outlined,
                    color: _primary,
                    size: 32,
                  ),
                  border: InputBorder.none,
                  contentPadding: EdgeInsets.symmetric(vertical: 16),
                ),
              ),
            ),
          ),
          const SizedBox(width: 10),
          IconButton(
            tooltip: 'More',
            icon: const Icon(Icons.more_vert, color: _primary, size: 32),
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}

class _SearchLanding extends StatelessWidget {
  const _SearchLanding();

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 10, 16, 24),
      children: const [
        _SearchChips(),
        SizedBox(height: 92),
        _SearchIllustration(),
        SizedBox(height: 20),
        Center(
          child: Text(
            'Search for your photos and videos',
            style: TextStyle(
              fontSize: 21,
              fontWeight: FontWeight.w700,
              color: _text,
            ),
            textAlign: TextAlign.center,
          ),
        ),
        SizedBox(height: 44),
        _QuickSearchPanel(),
      ],
    );
  }
}

class _SearchChips extends StatelessWidget {
  const _SearchChips();

  @override
  Widget build(BuildContext context) {
    const chips = [
      (Icons.groups_outlined, 'People'),
      (Icons.location_on_outlined, 'Location'),
      (Icons.photo_camera_outlined, 'Camera'),
      (Icons.calendar_month_outlined, 'Date'),
    ];

    return SizedBox(
      height: 54,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: chips.length,
        separatorBuilder: (_, _) => const SizedBox(width: 12),
        itemBuilder: (context, index) {
          final (icon, label) = chips[index];
          return Container(
            padding: const EdgeInsets.symmetric(horizontal: 18),
            decoration: BoxDecoration(
              color: _pageColor,
              border: Border.all(color: const Color(0xFFE5E2EA)),
              borderRadius: BorderRadius.circular(27),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(icon, size: 24, color: _text),
                const SizedBox(width: 8),
                Text(label, style: const TextStyle(fontSize: 18, color: _text)),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _SearchIllustration extends StatelessWidget {
  const _SearchIllustration();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Container(
        width: 126,
        height: 126,
        decoration: BoxDecoration(
          color: const Color(0xFFE8E5FA),
          borderRadius: BorderRadius.circular(18),
        ),
        child: const Stack(
          alignment: Alignment.center,
          children: [
            Icon(Icons.photo_camera_outlined, color: _primary, size: 82),
            Positioned(
              bottom: 18,
              child: Icon(
                Icons.photo_outlined,
                color: Color(0xFFEF7F45),
                size: 42,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _QuickSearchPanel extends StatelessWidget {
  const _QuickSearchPanel();

  @override
  Widget build(BuildContext context) {
    const rows = [
      (Icons.access_time, 'Recently taken'),
      (Icons.upload_outlined, 'Recently added'),
      (Icons.play_circle_outline, 'Videos'),
      (Icons.favorite_border, 'Favorites'),
    ];

    return Container(
      decoration: BoxDecoration(
        color: _panelColor,
        borderRadius: BorderRadius.circular(30),
        border: Border.all(color: const Color(0xFFE7E3EE)),
      ),
      child: Column(
        children: [
          for (var i = 0; i < rows.length; i++)
            _QuickSearchRow(
              icon: rows[i].$1,
              label: rows[i].$2,
              showDivider: i != rows.length - 1,
            ),
        ],
      ),
    );
  }
}

class _QuickSearchRow extends StatelessWidget {
  const _QuickSearchRow({
    required this.icon,
    required this.label,
    required this.showDivider,
  });

  final IconData icon;
  final String label;
  final bool showDivider;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: () {},
      borderRadius: BorderRadius.circular(30),
      child: Container(
        height: 86,
        decoration: BoxDecoration(
          border: showDivider
              ? const Border(bottom: BorderSide(color: Color(0xFFE7E3EE)))
              : null,
        ),
        padding: const EdgeInsets.symmetric(horizontal: 28),
        child: Row(
          children: [
            Icon(icon, size: 32, color: const Color(0xFF55525D)),
            const SizedBox(width: 28),
            Text(
              label,
              style: const TextStyle(
                fontSize: 23,
                fontWeight: FontWeight.w700,
                color: _text,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SearchResults extends StatelessWidget {
  const _SearchResults({
    required this.future,
    required this.query,
    required this.onRetry,
  });

  final Future<List<Asset>> future;
  final String query;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: future,
      builder: (context, snapshot) {
        if (snapshot.connectionState != ConnectionState.done) {
          return const Center(child: CircularProgressIndicator());
        }
        if (snapshot.hasError) {
          return _SearchErrorView(onRetry: onRetry);
        }
        final assets = snapshot.data ?? const [];
        if (assets.isEmpty) {
          return Center(
            child: Text(
              'No results for "$query"',
              style: Theme.of(context).textTheme.titleMedium,
            ),
          );
        }
        return CustomScrollView(
          slivers: [
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
                child: Text(
                  'Search results',
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
            Text(
              'Search is unavailable',
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
