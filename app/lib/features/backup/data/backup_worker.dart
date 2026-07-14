import 'package:dio/dio.dart';
import 'package:flutter/widgets.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:workmanager/workmanager.dart';

import 'backup_repository.dart';
import 'backup_settings.dart';

@pragma('vm:entry-point')
void callbackDispatcher() {
  Workmanager().executeTask((taskName, inputData) async {
    WidgetsFlutterBinding.ensureInitialized();
    if (taskName != domusBackupTaskName) return true;
    final prefs = await SharedPreferences.getInstance();
    final serverUrl = prefs.getString('serverUrl');
    final token = prefs.getString('accessToken');
    if (serverUrl == null || token == null) return true;
    final dio = Dio(
      BaseOptions(
        baseUrl: '${serverUrl.replaceAll(RegExp(r'/+$'), '')}/api',
        connectTimeout: const Duration(seconds: 20),
        receiveTimeout: const Duration(minutes: 10),
        headers: {'Authorization': 'Bearer $token'},
      ),
    );
    await runAutoBackupWithClient(dio);
    return true;
  });
}

Future<void> initializeBackgroundBackup() async {
  await Workmanager().initialize(callbackDispatcher);
}
