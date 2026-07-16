import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../data/immich_derivative_repository.dart';

class ImmichDerivativePanel extends ConsumerStatefulWidget {
  const ImmichDerivativePanel({super.key});

  @override
  ConsumerState<ImmichDerivativePanel> createState() =>
      _ImmichDerivativePanelState();
}

class _ImmichDerivativePanelState extends ConsumerState<ImmichDerivativePanel> {
  final _limit = TextEditingController(text: '200');
  final _shardIndex = TextEditingController(text: '0');
  final _shardCount = TextEditingController(text: '1');
  final _assetIds = TextEditingController();
  bool _repairAll = false;
  bool _resume = true;

  @override
  void dispose() {
    _limit.dispose();
    _shardIndex.dispose();
    _shardCount.dispose();
    _assetIds.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final statusValue = ref.watch(immichDerivativeStatusProvider);
    return statusValue.when(
      loading: () => const ListTile(
        leading: Icon(Icons.photo_filter_outlined),
        title: Text('Immich derivatives'),
        subtitle: LinearProgressIndicator(),
      ),
      error: (error, _) => ListTile(
        leading: const Icon(Icons.photo_filter_outlined),
        title: const Text('Immich derivatives'),
        subtitle: Text('$error'),
        trailing: IconButton(
          icon: const Icon(Icons.refresh),
          onPressed: () => ref.invalidate(immichDerivativeStatusProvider),
        ),
      ),
      data: (status) => ExpansionTile(
        leading: const Icon(Icons.photo_filter_outlined),
        title: const Text('Immich derivatives'),
        subtitle: Text(_subtitle(status)),
        childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
        children: [
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              SizedBox(
                width: 120,
                child: TextField(
                  controller: _limit,
                  enabled: !status.running && !_repairAll,
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  decoration: const InputDecoration(labelText: 'Limit'),
                ),
              ),
              SizedBox(
                width: 120,
                child: TextField(
                  controller: _shardIndex,
                  enabled: !status.running,
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  decoration: const InputDecoration(labelText: 'Shard'),
                ),
              ),
              SizedBox(
                width: 120,
                child: TextField(
                  controller: _shardCount,
                  enabled: !status.running,
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  decoration: const InputDecoration(labelText: 'Shards'),
                ),
              ),
            ],
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            value: _repairAll,
            onChanged: status.running
                ? null
                : (value) => setState(() => _repairAll = value),
            title: const Text('Repair all assets'),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            value: _resume,
            onChanged: status.running
                ? null
                : (value) => setState(() => _resume = value),
            title: const Text('Resume completed assets'),
          ),
          TextField(
            controller: _assetIds,
            enabled: !status.running,
            minLines: 1,
            maxLines: 3,
            decoration: const InputDecoration(
              labelText: 'Asset IDs',
              hintText: 'Comma or newline separated',
            ),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              FilledButton.icon(
                icon: const Icon(Icons.play_arrow),
                label: const Text('Run'),
                onPressed: status.running ? null : _run,
              ),
              const SizedBox(width: 8),
              OutlinedButton.icon(
                icon: const Icon(Icons.stop),
                label: const Text('Cancel'),
                onPressed: status.running ? _cancel : null,
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.refresh),
                onPressed: () => ref.invalidate(immichDerivativeStatusProvider),
              ),
            ],
          ),
          const SizedBox(height: 12),
          LinearProgressIndicator(
            value: status.progress.total == 0
                ? null
                : (status.progress.checked / status.progress.total).clamp(
                    0.0,
                    1.0,
                  ),
          ),
          const SizedBox(height: 8),
          Align(alignment: Alignment.centerLeft, child: Text(_details(status))),
          if (status.progress.recentMessages.isNotEmpty) ...[
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerLeft,
              child: Text(
                status.progress.recentMessages.reversed.take(8).join('\n'),
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
          ],
        ],
      ),
    );
  }

  String _subtitle(ImmichDerivativeStatus status) {
    if (status.running) {
      return '${status.progress.phase}: ${status.progress.checked}/${status.progress.total}';
    }
    if (status.error != null) return 'Failed: ${status.error}';
    final summary = status.summary;
    if (summary != null) {
      return '${summary.cancelled ? 'Cancelled' : 'Done'}: ${summary.ok} ok, ${summary.failed} failed, ${summary.resumed} resumed';
    }
    return 'Repair thumbnails, previews, fullsize images, and video covers';
  }

  String _details(ImmichDerivativeStatus status) {
    final lines = <String>[
      'Phase: ${status.progress.phase}',
      'Checked: ${status.progress.checked}/${status.progress.total}  OK: ${status.progress.ok}  Failed: ${status.progress.failed}  Resumed: ${status.progress.resumed}',
    ];
    if (status.progress.currentAssetId != null) {
      lines.add('Asset: ${status.progress.currentAssetId}');
    }
    if (status.error != null) lines.add('Error: ${status.error}');
    final summary = status.summary;
    if (summary != null) {
      lines.add(
        'Summary: ${summary.ok} ok, ${summary.failed} failed, ${summary.resumed} resumed',
      );
    }
    return lines.join('\n');
  }

  Future<void> _run() async {
    final request = ImmichDerivativeRequest(
      limit: int.tryParse(_limit.text) ?? 200,
      repairAll: _repairAll,
      shardIndex: int.tryParse(_shardIndex.text) ?? 0,
      shardCount: int.tryParse(_shardCount.text) ?? 1,
      resume: _resume,
      assetIds: _assetIds.text
          .split(RegExp(r'[,\s]+'))
          .map((value) => value.trim())
          .where((value) => value.isNotEmpty)
          .toList(),
    );
    await ref.read(immichDerivativeRepositoryProvider).run(request);
    ref.invalidate(immichDerivativeStatusProvider);
  }

  Future<void> _cancel() async {
    await ref.read(immichDerivativeRepositoryProvider).cancel();
    ref.invalidate(immichDerivativeStatusProvider);
  }
}
