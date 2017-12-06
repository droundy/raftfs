RaftFS
======

This project is at an early stage.  The goal is a posix-compliant file
system which mirrors its content over every computer (node) that
mounts it.  Reads will be fast and will not require network access.
Writes of any sort will be slow, and will use raft to ensure that they
are coherent across the cluster.  The result should be a
fault-tolerant filesystem with (hopefully) superior performance to
NFS, which should scale up to a half dozen or so nodes.

The data will be stored as ordinary files on an existing filesystem,
so in case of disaster (e.g., all but one node are hit by a meteor),
you have your filesystem there on disk, and don't even need to use
RaftFS to access it.  This mimics one advantage of NFS:  you can
export an existing directory, and do not need to mess with
partitioning or block devices when you decide to share a directory.

Status
------

Currently, I have taken a demo passthrough filesystem, and am working
on enabling snapshotting ability.  This is needed in order to enable a
new node to join the cluster, since it will first need to get a copy
of the state of the filesystem at one moment in time (and then apply
all subsequent changes).

You can take a snapshot manually by creating a directory called
.snapshots/NAME (where NAME is the name of your snapshot) in your
mounted filesystem.  This creates a snapshot, which should be
read-only and exist in that directory.

Using it
--------

To use it and test fuse_mt, run:

    cargo run <path to filesystem> <mount point>

where `<path to filesystem>` is an existing directory containing files
and directories, and `<mount point>` is an empty directory where you
want your filesystem to be mounted.  Unmount it with `fusermount -u
<mount point>` or just CTRL-C the running program.
