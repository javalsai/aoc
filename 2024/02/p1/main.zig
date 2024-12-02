const std = @import("std");

pub fn main() !void {
    var file = try std.fs.cwd()
        .openFile("../input.txt", .{});
    defer file.close();

    var buf_reader = std.io.bufferedReader(file.reader());
    var in_stream = buf_reader.reader();

    var buf: [64]u8 = undefined;
    var t: u32 = 0;
    while (try in_stream.readUntilDelimiterOrEof(&buf, '\n')) |line| {
        var split_iter = std.mem.split(u8, line, " ");

        var dir: i8 = 0;
        var prev = try std.fmt.parseInt(i8, split_iter.next() orelse unreachable, 10);
        t += while (split_iter.next()) |strnum| {
            const n = try std.fmt.parseInt(i8, strnum, 10);
            const d = prev - n;
            prev = n;
            if (d == 0 or @abs(d) > 3)
                break 0;

            if (dir == 0) {
                dir = d;
            } else if (!sameSign(dir, d)) {
                break 0;
            }
        } else 1;
    }
    std.debug.print("Total {d}\n", .{t});
}

pub fn sameSign(n1: i8, n2: i8) bool {
    return (n1 < 0) == (n2 < 0);
}
