import os
import time
from watchdog.events import FileSystemEventHandler
from watchdog.observers import Observer


def watch_note(note, handle_func):
    """watch a single note for changes,
    call `handle_func` on change"""
    ob = Observer()
    handler = FileSystemEventHandler()

    def handle_event(event):
        """update the note only if:
            - the note itself is changed
            - a referenced asset is changed
        """
        _, filename = os.path.split(event.src_path)

        # get all absolute paths of referenced images
        images = []
        for img_path in note.images:
            if not os.path.isabs(img_path):
                img_path = os.path.join(note.dir, img_path)
            images.append(img_path)

        if note.filename == filename or event.src_path in images:
            handle_func(note)
    handler.on_any_event = handle_event

    print('watching {0}...'.format(note.title))
    ob.schedule(handler, note.dir, recursive=True)
    ob.start()

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print('stopping...')
        ob.stop()
    ob.join()
