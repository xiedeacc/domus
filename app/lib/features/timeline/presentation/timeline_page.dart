import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';

import '../application/timeline_filter.dart';
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
  TimelineGranularity _granularity = TimelineGranularity.day;
  TimelineDateRange? _range;
  bool _showDateControls = false;
  Timer? _hideDateControlsTimer;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_handleScroll);
  }

  @override
  void dispose() {
    _hideDateControlsTimer?.cancel();
    _scrollController.removeListener(_handleScroll);
    _scrollController.dispose();
    super.dispose();
  }

  void _handleScroll() {
    if (!_showDateControls) {
      setState(() => _showDateControls = true);
    }
    _hideDateControlsTimer?.cancel();
    _hideDateControlsTimer = Timer(const Duration(seconds: 2), () {
      if (mounted && _range == null) {
        setState(() => _showDateControls = false);
      }
    });
  }

  Future<void> _pickRange() async {
    final now = DateTime.now();
    final initial = _range == null
        ? DateTimeRange(
            start: DateTime(now.year, now.month, now.day),
            end: DateTime(now.year, now.month, now.day),
          )
        : DateTimeRange(start: _range!.start, end: _range!.end);
    final selected = await showDateRangePicker(
      context: context,
      firstDate: DateTime(1970),
      lastDate: DateTime(now.year + 1, 12, 31),
      initialDateRange: initial,
    );
    if (selected == null || !mounted) return;
    setState(() => _range = normalizeDateRange(selected, _granularity));
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
        data: (buckets) {
          final visibleBuckets = _range == null
              ? buckets
              : buckets
                    .where(
                      (bucket) =>
                          _range!.overlapsMonthBucket(bucket.timeBucket),
                    )
                    .toList();
          return buckets.isEmpty
              ? const Center(child: Text('No photos yet'))
              : Stack(
                  children: [
                    Scrollbar(
                      controller: _scrollController,
                      thumbVisibility: true,
                      interactive: true,
                      child: ListView.builder(
                        controller: _scrollController,
                        padding: const EdgeInsets.only(bottom: 96),
                        itemCount: visibleBuckets.length,
                        itemBuilder: (context, index) => _BucketSection(
                          bucket: visibleBuckets[index].timeBucket,
                          range: _range,
                        ),
                      ),
                    ),
                    Positioned(
                      left: 12,
                      right: 12,
                      bottom: 12,
                      child: AnimatedOpacity(
                        opacity: _showDateControls || _range != null ? 1 : 0,
                        duration: const Duration(milliseconds: 180),
                        child: IgnorePointer(
                          ignoring: !_showDateControls && _range == null,
                          child: _TimelineDateControls(
                            granularity: _granularity,
                            range: _range,
                            onGranularityChanged: (value) {
                              setState(() {
                                _granularity = value;
                                final range = _range;
                                if (range != null) {
                                  _range = normalizeDateRange(
                                    DateTimeRange(
                                      start: range.start,
                                      end: range.end,
                                    ),
                                    value,
                                  );
                                }
                              });
                            },
                            onPickRange: _pickRange,
                            onClearRange: () => setState(() => _range = null),
                          ),
                        ),
                      ),
                    ),
                  ],
                );
        },
      ),
    );
  }
}

class _TimelineDateControls extends StatelessWidget {
  const _TimelineDateControls({
    required this.granularity,
    required this.range,
    required this.onGranularityChanged,
    required this.onPickRange,
    required this.onClearRange,
  });

  final TimelineGranularity granularity;
  final TimelineDateRange? range;
  final ValueChanged<TimelineGranularity> onGranularityChanged;
  final VoidCallback onPickRange;
  final VoidCallback onClearRange;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: colors.outlineVariant),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.14),
            blurRadius: 16,
            offset: const Offset(0, 6),
          ),
        ],
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final granularityPicker = SegmentedButton<TimelineGranularity>(
              segments: [
                for (final value in TimelineGranularity.values)
                  ButtonSegment(value: value, label: Text(value.label)),
              ],
              selected: {granularity},
              onSelectionChanged: (values) {
                onGranularityChanged(values.single);
              },
              showSelectedIcon: false,
            );
            final rangeButton = TextButton.icon(
              onPressed: onPickRange,
              icon: const Icon(Icons.date_range_outlined),
              label: Text(
                range?.label(granularity) ?? 'All dates',
                overflow: TextOverflow.ellipsis,
              ),
            );
            final clearButton = IconButton(
              tooltip: 'Clear',
              onPressed: range == null ? null : onClearRange,
              icon: const Icon(Icons.close),
            );
            if (constraints.maxWidth < 420) {
              return Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  granularityPicker,
                  Row(
                    children: [
                      Expanded(child: rangeButton),
                      clearButton,
                    ],
                  ),
                ],
              );
            }
            return Row(
              children: [
                granularityPicker,
                const SizedBox(width: 8),
                Expanded(child: rangeButton),
                clearButton,
              ],
            );
          },
        ),
      ),
    );
  }
}

class _BucketSection extends ConsumerWidget {
  const _BucketSection({required this.bucket, required this.range});

  final String bucket;
  final TimelineDateRange? range;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final assets = ref.watch(bucketAssetsProvider(bucket));
    final title = DateFormat.yMMMM().format(
      DateTime.tryParse(bucket) ?? DateTime.now(),
    );

    return assets.when(
      loading: () => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 16, 12, 8),
            child: Text(title, style: Theme.of(context).textTheme.titleMedium),
          ),
          const Padding(
            padding: EdgeInsets.all(24),
            child: Center(child: CircularProgressIndicator()),
          ),
        ],
      ),
      error: (e, _) =>
          Padding(padding: const EdgeInsets.all(12), child: Text('$e')),
      data: (assets) {
        final visibleAssets = range == null
            ? assets
            : assets.where(range!.includesAsset).toList();
        if (visibleAssets.isEmpty) return const SizedBox.shrink();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 16, 12, 8),
              child: Text(
                title,
                style: Theme.of(context).textTheme.titleMedium,
              ),
            ),
            GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 160,
                mainAxisSpacing: 2,
                crossAxisSpacing: 2,
              ),
              itemCount: visibleAssets.length,
              itemBuilder: (context, i) => AssetThumbnail(
                asset: visibleAssets[i],
                onTap: () => context.push('/asset/${visibleAssets[i].id}'),
              ),
            ),
          ],
        );
      },
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
