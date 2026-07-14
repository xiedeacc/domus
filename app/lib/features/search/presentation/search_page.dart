import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Search: metadata filters (filename, city, camera, date) backed by
/// POST /search/metadata. Smart (CLIP) search is a server feature Domus
/// does not provide — the tab is hidden when /server/features reports it off.
class SearchPage extends ConsumerStatefulWidget {
  const SearchPage({super.key});

  @override
  ConsumerState<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends ConsumerState<SearchPage> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: TextField(
          controller: _controller,
          decoration: const InputDecoration(
            hintText: 'Search your photos',
            border: InputBorder.none,
          ),
          onSubmitted: (query) {
            // TODO: POST /search/metadata with {originalFileName: query}
            // and render a result grid.
          },
        ),
      ),
      body: const Center(child: Text('Search by filename, place or date')),
    );
  }
}
