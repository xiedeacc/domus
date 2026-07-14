import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Backup control panel.
///
/// Mobile (Android/iOS): device photo library access + background uploads —
/// platform-channel work (photo_manager + native background tasks), see the
/// system design doc. Web: manual folder/file picker uploads only.
class BackupPage extends ConsumerWidget {
  const BackupPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('Backup')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: const [
          Card(
            child: ListTile(
              leading: Icon(Icons.cloud_upload_outlined),
              title: Text('Automatic backup'),
              subtitle: Text(
                  'Select device albums to back up automatically (mobile only)'),
              trailing: Switch(value: false, onChanged: null),
            ),
          ),
          SizedBox(height: 8),
          Card(
            child: ListTile(
              leading: Icon(Icons.upload_file_outlined),
              title: Text('Upload files'),
              subtitle: Text('Pick photos or folders to upload now'),
            ),
          ),
        ],
      ),
    );
  }
}
