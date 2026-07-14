import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';

import '../application/timeline_provider.dart';
import '../widgets/asset_thumbnail.dart';

/// Photo timeline: one section per month bucket, thumbnails in a grid.
/// The full implementation virtualizes buckets and adds a drag scrubber;
/// the skeleton renders eagerly.
class TimelinePage extends ConsumerStatefulWidget {
  const TimelinePage({super.key});

  @override
  ConsumerState<TimelinePage> createState() => _TimelinePageState();
}

class _TimelinePageState extends ConsumerState<TimelinePage> {
  final _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final buckets = ref.watch(timeBucketsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Photos'),
        actions: [
          IconButton(
            icon: const Icon(Icons.backup_outlined),
            onPressed: () => context.push('/backup'),
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => context.push('/settings'),
          ),
        ],
      ),
      body: buckets.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => _ErrorView(message: '$e'),
        data: (buckets) => buckets.isEmpty
            ? const Center(child: Text('No photos yet'))
            : Scrollbar(
                controller: _scrollController,
                thumbVisibility: true,
                interactive: true,
                child: ListView.builder(
                  controller: _scrollController,
                  itemCount: buckets.length,
                  itemBuilder: (context, index) =>
                      _BucketSection(bucket: buckets[index].timeBucket),
                ),
              ),
      ),
    );
  }
}

class _BucketSection extends ConsumerWidget {
  const _BucketSection({required this.bucket});

  final String bucket;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final assets = ref.watch(bucketAssetsProvider(bucket));
    final title = DateFormat.yMMMM().format(
      DateTime.tryParse(bucket) ?? DateTime.now(),
    );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 16, 12, 8),
          child: Text(title, style: Theme.of(context).textTheme.titleMedium),
        ),
        assets.when(
          loading: () => const Padding(
            padding: EdgeInsets.all(24),
            child: Center(child: CircularProgressIndicator()),
          ),
          error: (e, _) =>
              Padding(padding: const EdgeInsets.all(12), child: Text('$e')),
          data: (assets) => GridView.builder(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
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
      ],
    );
  }
}

class _ErrorView extends StatelessWidget {
  const _ErrorView({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Text(message, textAlign: TextAlign.center),
      ),
    );
  }
}
