const std = @import("std");

pub fn main() !void {
    const line = "50 29 30 31 34 35 37";
    var iter = std.mem.split(u8, line, " ");
    const res = check_list(&iter, 0);
    std.debug.print("{any}\n", .{res});
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
        std.debug.print("i={d} prev={} sk={any} n={}\n", .{ i, prev, skip, n });
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
