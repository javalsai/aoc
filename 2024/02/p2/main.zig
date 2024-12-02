const std = @import("std");

pub fn main() !void {
    var file = try std.fs.cwd()
        .openFile("../input.txt", .{});
    defer file.close();

    var buf_reader = std.io.bufferedReader(file.reader());
    var in_stream = buf_reader.reader();

    var buf: [64]u8 = undefined;
    var t: u32 = 0;
    out: while (try in_stream.readUntilDelimiterOrEof(&buf, '\n')) |line| {
        var split_iter = std.mem.split(u8, line, " ");
        const first_res = try check_list(&split_iter, null);
        if (first_res != null) {
            var res: ?usize = first_res;
            var fail: usize = 0;
            while (res != null) {
                if (res.? < fail) {
                    // std.debug.print(".. {s}\n", .{line});
                    continue :out;
                }
                var split_iter_l = std.mem.split(u8, line, " ");
                res = try check_list(&split_iter_l, fail);
                fail += 1;
            }
        }

        t += 1;
    }
    std.debug.print("Total {d}\n", .{t});
}

pub fn sameSign(n1: i8, n2: i8) bool {
    return (n1 < 0) == (n2 < 0);
}

// null if everythin alr
// idx num of failure otherwise
pub fn check_list(iter: *std.mem.SplitIterator(u8, .sequence), skip: ?usize) !?usize {
    var dir: i8 = 0;
    var i: usize = 0;
    if (skip != null and skip.? == 0) {
        _ = iter.next();
        i = 1;
    }
    var prev = try std.fmt.parseInt(i8, iter.next().?, 10);
    while (iter.next()) |strnum| {
        i += 1;
        if (skip != null and skip.? == i) {
            continue;
        }
        const n = try std.fmt.parseInt(i8, strnum, 10);
        // std.debug.print("i={d} prev={} sk={any} n={}\n", .{ i, prev, skip, n });
        const d = prev - n;
        if (d == 0 or @abs(d) > 3) {
            return i;
        }

        if (dir == 0) {
            dir = d;
        } else if (!sameSign(dir, d)) {
            return i;
        }
        prev = n;
    }
    return null;
}
