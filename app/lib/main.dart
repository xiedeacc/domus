import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'features/backup/data/backup_worker.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  try {
    await initializeBackgroundBackup();
  } catch (_) {
    // Workmanager is only available on Android/iOS.
  }
  runApp(const ProviderScope(child: DomusApp()));
}
