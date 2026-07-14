import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';

import '../../timeline/application/timeline_provider.dart';
import '../data/backup_repository.dart';
import '../data/backup_settings.dart';

/// Backup control panel.
///
/// Mobile (Android/iOS): device photo library access + background uploads —
/// platform-channel work (photo_manager + native background tasks), see the
/// system design doc. Web: manual folder/file picker uploads only.
class BackupPage extends ConsumerStatefulWidget {
  const BackupPage({super.key});

  @override
  ConsumerState<BackupPage> createState() => _BackupPageState();
}

class _BackupPageState extends ConsumerState<BackupPage> {
  bool _uploading = false;
  double? _progress;
  String? _message;

  @override
  Widget build(BuildContext context) {
    final settings = ref.watch(backupSettingsProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Backup')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: settings.when(
              loading: () => const ListTile(
                leading: CircularProgressIndicator(),
                title: Text('Loading backup settings'),
              ),
              error: (error, _) => ListTile(
                leading: const Icon(Icons.error_outline),
                title: const Text('Backup settings unavailable'),
                subtitle: Text('$error'),
              ),
              data: (settings) => Column(
                children: [
                  SwitchListTile(
                    secondary: const Icon(Icons.cloud_upload_outlined),
                    value: settings.autoBackup,
                    onChanged: (value) => ref
                        .read(backupSettingsProvider.notifier)
                        .setAutoBackup(value),
                    title: const Text('Automatic backup'),
                    subtitle: Text(
                      settings.selectedAlbumIds.isEmpty
                          ? 'Choose albums to upload when the app opens'
                          : '${settings.selectedAlbumIds.length} album(s) selected',
                    ),
                  ),
                  SwitchListTile(
                    secondary: const Icon(Icons.sync_outlined),
                    value: settings.backgroundBackup,
                    onChanged: (value) => ref
                        .read(backupSettingsProvider.notifier)
                        .setBackgroundBackup(value),
                    title: const Text('Background backup'),
                    subtitle: const Text('Runs periodically while connected'),
                  ),
                  ListTile(
                    leading: const Icon(Icons.photo_library_outlined),
                    title: const Text('Selected albums'),
                    subtitle: Text(_albumSelectionLabel(settings)),
                    trailing: TextButton(
                      onPressed: _selectAlbums,
                      child: const Text('Choose'),
                    ),
                  ),
                  ListTile(
                    leading: const Icon(Icons.play_arrow_outlined),
                    title: const Text('Run backup now'),
                    subtitle: Text(
                      _message ?? 'Upload new assets from selected albums',
                    ),
                    trailing: FilledButton(
                      onPressed: _uploading ? null : _runAutoBackup,
                      child: const Text('Run'),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 8),
          Card(
            child: ListTile(
              leading: _uploading
                  ? const SizedBox.square(
                      dimension: 24,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.upload_file_outlined),
              title: const Text('Upload files'),
              subtitle: Text(_message ?? 'Pick photos or videos to upload now'),
              trailing: FilledButton.icon(
                onPressed: _uploading ? null : _pickAndUpload,
                icon: const Icon(Icons.add_photo_alternate_outlined),
                label: const Text('Select'),
              ),
            ),
          ),
          if (_progress != null) ...[
            const SizedBox(height: 12),
            LinearProgressIndicator(value: _progress),
          ],
        ],
      ),
    );
  }

  String _albumSelectionLabel(BackupSettings settings) {
    final count = settings.selectedAlbumIds.length;
    if (count == 0) return 'None';
    if (count == 1) return '1 album selected';
    return '$count albums selected';
  }

  Future<void> _selectAlbums() async {
    final albums = await ref.read(backupRepositoryProvider).albums();
    final settings = ref.read(backupSettingsProvider).value;
    final selected = {...?settings?.selectedAlbumIds};
    if (!mounted) return;
    final result = await showDialog<List<String>>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('Backup albums'),
          content: SizedBox(
            width: 420,
            child: ListView(
              shrinkWrap: true,
              children: [
                for (final album in albums)
                  CheckboxListTile(
                    value: selected.contains(album.id),
                    onChanged: (value) {
                      setDialogState(() {
                        if (value == true) {
                          selected.add(album.id);
                        } else {
                          selected.remove(album.id);
                        }
                      });
                    },
                    title: Text(album.name),
                    subtitle: Text('${album.count} asset(s)'),
                  ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(selected.toList()),
              child: const Text('Save'),
            ),
          ],
        ),
      ),
    );
    if (result == null) return;
    await ref.read(backupSettingsProvider.notifier).setSelectedAlbumIds(result);
  }

  Future<void> _runAutoBackup() async {
    setState(() {
      _uploading = true;
      _message = 'Scanning selected albums';
      _progress = null;
    });
    try {
      final count = await ref.read(backupRepositoryProvider).runAutoBackup();
      ref.invalidate(timeBucketsProvider);
      setState(() => _message = 'Uploaded $count new asset(s)');
    } catch (error) {
      setState(() => _message = 'Backup failed: $error');
    } finally {
      if (mounted) setState(() => _uploading = false);
    }
  }

  Future<void> _pickAndUpload() async {
    final result = await FilePicker.platform.pickFiles(
      allowMultiple: true,
      type: FileType.media,
      withData: true,
    );
    if (result == null || result.files.isEmpty) {
      return;
    }
    setState(() {
      _uploading = true;
      _progress = null;
      _message = 'Uploading ${result.files.length} file(s)';
    });
    try {
      final uploaded = await ref
          .read(backupRepositoryProvider)
          .uploadPickedFiles(
            result.files,
            onProgress: (sent, total) {
              if (total > 0 && mounted) {
                setState(() => _progress = sent / total);
              }
            },
          );
      ref.invalidate(timeBucketsProvider);
      setState(() {
        _message = 'Uploaded ${uploaded.length} file(s)';
        _progress = null;
      });
    } catch (error) {
      setState(() {
        _message = 'Upload failed: $error';
        _progress = null;
      });
    } finally {
      if (mounted) {
        setState(() => _uploading = false);
      }
    }
  }
}
